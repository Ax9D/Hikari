use std::sync::Arc;

use hikari_3d::*;
use hikari_asset::AssetPool;
use hikari_math::*;
use hikari_render::*;

use crate::{Args, light::CascadeRenderInfo};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
struct Material {
    albedo: hikari_math::Vec4,
    roughness: f32,
    metallic: f32,
    albedo_uv_set: i32,
    roughness_uv_set: i32,
    metallic_uv_set: i32,
    normal_uv_set: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct PushConstants {
    transform: hikari_math::Mat4,
    material_data: Material,
}
struct Defaults {
    default_mat: hikari_3d::Material,
    checkerboard: Texture2D,
    black: Texture2D,
}
impl Defaults {
    pub fn prepare(device: &Arc<Device>) -> Self {
        let (checkerboard, width, height) =
            hikari_3d::old::image::load_from_file("engine_assets/textures/checkerboard.png")
                .expect("Failed to load checkerboard texture");
        let checkerboard = Texture2D::new(
            device,
            &checkerboard,
            width,
            height,
            TextureConfig {
                format: Format::RGBA8,
                wrap_x: WrapMode::Repeat,
                wrap_y: WrapMode::Repeat,
                filtering: FilterMode::Linear,
                aniso_level: 9.0,
                generate_mips: true,
            },
        )
        .expect("Failed to create checkerboard texture");

        let black = Texture2D::new(
            device,
            &[0, 0, 0, 255],
            1,
            1,
            TextureConfig {
                format: Format::RGBA8,
                wrap_x: WrapMode::Repeat,
                wrap_y: WrapMode::Repeat,
                filtering: FilterMode::Linear,
                aniso_level: 0.0,
                generate_mips: false,
            },
        )
        .expect("Failed to create black texture");

        let default_mat = hikari_3d::Material::default();
        Self {
            black,
            checkerboard,
            default_mat,
        }
    }
}
fn resolve_texture<'a>(
    handle: &Option<hikari_asset::Handle<Texture2D>>,
    textures: &'a AssetPool<Texture2D>,
    default: &'a Texture2D,
) -> &'a Texture2D {
    handle
        .as_ref()
        .map(|handle| textures.get(handle).unwrap_or(default))
        .unwrap_or(default)
}

pub fn build_pass(
    device: &Arc<Device>,
    graph: &mut GraphBuilder<Args>,
    shader_lib: &mut ShaderLibrary,
    shadow_atlas: &GpuHandle<SampledImage>,
    cascade_render_buffer: &GpuHandle<GpuBuffer<CascadeRenderInfo>>,
    depth_prepass: &GpuHandle<SampledImage>
) -> anyhow::Result<GpuHandle<SampledImage>> {
    let defaults = Defaults::prepare(device);
    shader_lib.insert("pbr")?;
    
    let layout = VertexInputLayout::builder()
        .buffer(
            &[
                ShaderDataType::Vec3f,
            ],
            StepMode::Vertex,
        )
        .buffer(
            &[
                ShaderDataType::Vec3f,
            ],
            StepMode::Vertex,
        )
        .buffer(
            &[
                ShaderDataType::Vec2f,
            ],
            StepMode::Vertex,
        )
        .buffer(
            &[
                ShaderDataType::Vec2f,
            ],
            StepMode::Vertex,
        )
        .build();

    let mut config = ImageConfig::color2d();
    config.format = vk::Format::R16G16B16A16_SFLOAT;
    let color_output = graph
        .create_image("PBRColor", config, ImageSize::default_xy())
        .expect("Failed to create PBR attachments");

    
    let shadow_atlas = shadow_atlas.clone();
    let cascade_render_buffer = cascade_render_buffer.clone();

    let renderpass = Renderpass::<Args>::new(
        "PBR",
        ImageSize::default_xy())
    .read_image(&shadow_atlas, AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer)
    .read_buffer(&cascade_render_buffer, AccessType::FragmentShaderReadOther)
    .draw_image(&color_output, AttachmentConfig::color_default(0))
    .draw_image(
        &depth_prepass,
        AttachmentConfig {
            kind: AttachmentKind::DepthOnly,
            access: AccessType::DepthStencilAttachmentRead,
            load_op: hikari_render::vk::AttachmentLoadOp::LOAD,
            store_op: hikari_render::vk::AttachmentStoreOp::STORE,
            stencil_load_op: hikari_render::vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: hikari_render::vk::AttachmentStoreOp::DONT_CARE,
        },
    ).cmd(move |cmd, graph_res, record_info, (world, res, shader_lib, assets)| {
        cmd.set_image(graph_res.get_image(&shadow_atlas).unwrap(), 0, 1);
        
        let camera = res.camera;

        if camera.is_some() {
            cmd.set_viewport(0.0, 0.0, record_info.framebuffer_width as f32, record_info.framebuffer_height as f32);
            cmd.set_scissor(0, 0, record_info.framebuffer_width, record_info.framebuffer_height);

            cmd.set_shader(shader_lib.get("pbr").unwrap());

            cmd.set_vertex_input_layout(layout);

            cmd.set_depth_stencil_state(DepthStencilState {
                depth_test_enabled: true,
                depth_write_enabled: false,
                depth_compare_op: CompareOp::Equal,
                ..Default::default()
            });

            cmd.set_buffer(&res.world_ubo, 0..1, 0, 0);

            let cascade_render_buffer = graph_res.get_buffer(&cascade_render_buffer).unwrap();
            cmd.set_buffer(cascade_render_buffer, 0..cascade_render_buffer.len(), 0, 2);

            {
                hikari_dev::profile_scope!("Render scene");
                let scenes = assets.read_assets::<Scene>().expect("Meshes pool not found");
                let materials = assets
                    .read_assets::<hikari_3d::Material>()
                    .expect("Materials pool not found");
                let textures = assets.read_assets::<Texture2D>().expect("Textures pool not found");

                for (_, (transform, mesh_comp)) in
                    &mut world.query::<(&Transform, &MeshRender)>()
                {
                    let mut transform = transform.get_matrix();
                    match &mesh_comp.source {
                        MeshSource::Scene(handle, mesh_ix) => {
                            if let Some(scene) = scenes.get(handle) {
                                let mesh = &scene.meshes[*mesh_ix];

                                transform *= mesh.transform.get_matrix();

                                for submesh in &mesh.sub_meshes {
                                    {
                                        hikari_dev::profile_scope!(
                                            "Set vertex and index buffers"
                                        );
                                        cmd.set_vertex_buffers(&[&submesh.position, &submesh.normals, &submesh.tc0, &submesh.tc1], 0);

                                        cmd.set_index_buffer(&submesh.indices);
                                    }
                                    let material = materials
                                        .get(&submesh.material)
                                        .unwrap_or_else(|| &defaults.default_mat);

                                    let material_data = Material {
                                        albedo: material.albedo_factor,
                                        roughness: material.roughness_factor,
                                        metallic: material.metallic_factor,
                                        albedo_uv_set: material.albedo_set,
                                        roughness_uv_set: material.roughness_set,
                                        metallic_uv_set: material.metallic_set,
                                        normal_uv_set: material.normal_set,
                                    };

                                    let pc = PushConstants {
                                        transform,
                                        material_data,
                                    };

                                    cmd.push_constants(&pc, 0);

                                    let albedo = resolve_texture(
                                        &material.albedo,
                                        &textures,
                                        &defaults.checkerboard,
                                    );
                                    let roughness = resolve_texture(
                                        &material.roughness,
                                        &textures,
                                        &defaults.black,
                                    );
                                    let metallic = resolve_texture(
                                        &material.metallic,
                                        &textures,
                                        &defaults.black,
                                    );
                                    let normal = resolve_texture(
                                        &material.normal,
                                        &textures,
                                        &defaults.black,
                                    );

                                    // println!(
                                    //     "{:?} {:?} {:?} {:?}",
                                    //     albedo.raw().image(),
                                    //     roughness.raw().image(),
                                    //     metallic.raw().image(),
                                    //     normal.raw().image()
                                    // );
                                    cmd.set_image(albedo.raw(), 1, 0);
                                    cmd.set_image(roughness.raw(), 1, 1);
                                    cmd.set_image(metallic.raw(), 1, 2);
                                    cmd.set_image(normal.raw(), 1, 3);

                                    cmd.draw_indexed(
                                        0..submesh.indices.capacity(),
                                        0,
                                        0..1,
                                    );
                                }
                            }
                        }
                        MeshSource::None => {}
                    }
                }
            }
        } else {
            log::warn!("No camera in the world");
        }
    });

    // for (ix, cascade) in shadow_cascades.iter().enumerate() {
    //     renderpass = renderpass.sample_image_array(cascade, AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer, 1, ix);
    // }

    graph.add_renderpass(renderpass);

    Ok(color_output)
}
