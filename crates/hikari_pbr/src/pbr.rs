use std::sync::Arc;

use hikari_3d::*;
use hikari_asset::AssetPool;
use hikari_math::*;
use hikari_render::*;

use crate::util::*;
use crate::Args;

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

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct DirLight {
    intensity: f32,
    color: hikari_math::Vec3A,
    direction: hikari_math::Vec3A,
}
#[repr(C)]
#[derive(Copy, Clone, Default)]

struct UBO {
    camera_position: hikari_math::Vec3A,
    view_proj: [f32; 16],
    exposure: f32,

    dir_light: DirLight,
}

struct Defaults {
    default_mat: hikari_3d::Material,
    checkerboard: Texture2D,
    black: Texture2D,
}
impl Defaults {
    pub fn prepare(device: &Arc<Device>) -> Self {
        let (checkerboard, width, height) =
            hikari_3d::old::image::load_from_file("assets/textures/checkerboard.png")
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

fn load_shader(device: &Arc<Device>) -> anyhow::Result<Arc<Shader>> {
    let shader = ShaderProgramBuilder::vertex_and_fragment(
        "PBR",
        &ShaderCode {
            entry_point: "main",
            data: ShaderData::Glsl(std::fs::read_to_string("assets/shaders/pbr.vert")?),
        },
        &ShaderCode {
            entry_point: "main",
            data: ShaderData::Glsl(std::fs::read_to_string("assets/shaders/pbr.frag")?),
        },
    )
    .build(device)?;

    Ok(shader)
}
pub fn build_pass(
    device: &Arc<Device>,
    graph: &mut GraphBuilder<Args>,
    depth_prepass: &GpuHandle<SampledImage>,
) -> anyhow::Result<GpuHandle<SampledImage>> {
    log::debug!("sizeof(UBO)={}", std::mem::size_of::<UBO>());

    let defaults = Defaults::prepare(device);
    let shader = load_shader(device)?;

    let mut ubo = PerFrame::new([
        hikari_render::create_uniform_buffer::<UBO>(device, 1)?,
        hikari_render::create_uniform_buffer::<UBO>(device, 1)?,
    ]);
    let layout = VertexInputLayout::builder()
        .buffer(
            &[
                ShaderDataType::Vec3f,
                ShaderDataType::Vec3f,
                ShaderDataType::Vec2f,
                ShaderDataType::Vec2f,
            ],
            StepMode::Vertex,
        )
        .build();

    let color_output = graph
        .create_image("PBRColor", ImageConfig::color2d(), ImageSize::default())
        .expect("Failed to create PBR attachments");

    graph.add_renderpass(
        Renderpass::<Args>::new(
            "PBR",
            ImageSize::default(),
            move |cmd, (world, settings, assets)| {
                let camera = get_camera(world);
                if let Some(camera_entity) = camera {
                    let camera = world.get_component::<Camera>(camera_entity).unwrap();
                    let camera_transform = world.get_component::<Transform>(camera_entity).unwrap();

                    let proj = camera.get_projection_matrix(settings.width, settings.height);
                    let view = camera_transform.get_matrix().inverse();

                    let view_proj = (proj * view).to_cols_array();

                    let dir_light_entity = get_directional_light(world);
                    let dir_light_transform = dir_light_entity
                        .map(|entity| *world.get_component::<Transform>(entity).unwrap())
                        .unwrap_or_default();

                    let direction = dir_light_transform.rotation * -Vec3::Y;

                    let dir_light = dir_light_entity
                        .map(|entity| {
                            let light = world.get_component::<Light>(entity).unwrap();
                            DirLight {
                                color: light.color.into(),
                                direction: direction.into(),
                                intensity: light.intensity,
                            }
                        })
                        .unwrap_or_else(|| DirLight::default());

                    ubo.get_mut().mapped_slice_mut()[0] = UBO {
                        view_proj,
                        camera_position: camera_transform.position.into(),
                        exposure: camera.exposure,
                        dir_light: DirLight {
                            color: dir_light.color.into(),
                            direction: direction.into(),
                            intensity: dir_light.intensity,
                        },
                    };

                    cmd.set_shader(&shader);

                    cmd.set_vertex_input_layout(layout);

                    cmd.set_depth_stencil_state(DepthStencilState {
                        depth_test_enabled: true,
                        depth_write_enabled: false,
                        depth_compare_op: CompareOp::Equal,
                        ..Default::default()
                    });

                    cmd.set_uniform_buffer(ubo.get(), 0..1, 0, 0);

                    {
                        hikari_dev::profile_scope!("Render scene");
                        let scenes = assets.get::<Scene>().expect("Meshes pool not found");
                        let materials = assets
                            .get::<hikari_3d::Material>()
                            .expect("Materials pool not found");
                        let textures = assets.get::<Texture2D>().expect("Textures pool not found");

                        for (_, (transform, mesh_comp)) in
                            &mut world.query::<(&Transform, &MeshRender)>()
                        {
                            let transform = transform.get_matrix();
                            match &mesh_comp.source {
                                MeshSource::Scene(handle, mesh_ix) => {
                                    if let Some(scene) = scenes.get(handle) {
                                        let mesh = &scene.meshes[*mesh_ix];
                                        for submesh in &mesh.sub_meshes {
                                            {
                                                hikari_dev::profile_scope!(
                                                    "Set vertex and index buffers"
                                                );
                                                cmd.set_vertex_buffer(&submesh.vertices, 0);
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
                ubo.next_frame();
            },
        )
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
        ),
    );

    Ok(color_output)
}
