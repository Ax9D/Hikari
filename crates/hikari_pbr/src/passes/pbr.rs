use std::sync::Arc;

use hikari_3d::{primitives::Primitives, *};
use hikari_asset::{AssetManager, AssetPool, PoolRef};
use hikari_core::World;
use hikari_math::*;
use hikari_render::*;

use crate::{light::CascadeRenderInfo, resources::RenderResources, Args};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
struct MaterialPC {
    albedo: hikari_math::Vec4,
    emissive: Vec3,
    roughness: f32,
    metallic: f32,
    uv_set: u32,
    textures_mask: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct PushConstants {
    view_transform: hikari_math::Mat4,
    material_data: MaterialPC,
}
struct PBRPass {
    layout: VertexInputLayout,
    skybox_layout: VertexInputLayout,
    primitives: Arc<Primitives>,
    shadow_atlas: GpuHandle<SampledImage>,
    cascade_render_buffer: GpuHandle<GpuBuffer<CascadeRenderInfo>>,
}
impl PBRPass {
    pub fn build(
        graph: &mut GraphBuilder<Args>,
        shader_lib: &mut ShaderLibrary,
        primitives: &Arc<hikari_3d::primitives::Primitives>,
        shadow_atlas: &GpuHandle<SampledImage>,
        cascade_render_buffer: &GpuHandle<GpuBuffer<CascadeRenderInfo>>,
        depth_prepass: &GpuHandle<SampledImage>,
    ) -> anyhow::Result<GpuHandle<SampledImage>> {
        let layout = VertexInputLayout::builder()
            .buffer(&[ShaderDataType::Vec3f], StepMode::Vertex)
            .buffer(&[ShaderDataType::Vec3f], StepMode::Vertex)
            .buffer(&[ShaderDataType::Vec2f], StepMode::Vertex)
            .buffer(&[ShaderDataType::Vec2f], StepMode::Vertex)
            .build();

        let skybox_layout = VertexInputLayout::builder()
            .buffer(&[ShaderDataType::Vec3f], StepMode::Vertex)
            .build();

        shader_lib.insert("pbr")?;
        shader_lib.insert("unlit")?;
        shader_lib.insert("skybox")?;

        let primitives = primitives.clone();
        let mut config = ImageConfig::color2d_attachment();
        config.format = vk::Format::R16G16B16A16_SFLOAT;
        let color_output = graph
            .create_image("PBRColor", config, ImageSize::default_xy())
            .expect("Failed to create PBR attachments");

        let shadow_atlas = shadow_atlas.clone();
        let cascade_render_buffer = cascade_render_buffer.clone();

        let mut renderer = Self {
            layout,
            skybox_layout,
            primitives,
            shadow_atlas,
            cascade_render_buffer,
        };

        let renderpass = Renderpass::<Args>::new("PBR", ImageSize::default_xy())
            .read_image(
                &shadow_atlas,
                AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
            )
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
            )
            .cmd(
                move |cmd, graph_res, record_info, (world, res, shader_lib, asset_manager)| {
                    renderer.render(
                        cmd,
                        world,
                        asset_manager,
                        graph_res,
                        res,
                        record_info,
                        shader_lib,
                    );
                },
            );

        graph.add_renderpass(renderpass);

        Ok(color_output)
    }
    fn prepare_ibl(
        &self,
        cmd: &mut RenderpassCommands,
        environment: Option<(&Environment, &Transform)>,
        assets: &Assets,
    ) {
        let primitives = &self.primitives;
        let environment_textures = &assets.environment_textures;

        cmd.set_image(&primitives.black_cube.raw(), 0, 3);
        cmd.set_image(&primitives.black_cube.raw(), 0, 4);
        cmd.set_image(&primitives.brdf_lut, 2, 0);

        if let Some((environment, _transform)) = environment {
            if let Some(handle) = &environment.texture {
                if let Some(environment_texture) = environment_textures.get(handle) {
                    cmd.set_image(environment_texture.diffuse_irradiance(), 0, 3);
                    cmd.set_image(environment_texture.specular_prefiltered(), 0, 4);
                }
            }
        }
    }
    fn render_skybox(
        &self,
        cmd: &mut RenderpassCommands,
        environment: Option<(&Environment, &Transform)>,
        assets: &Assets,
        res: &RenderResources,
        shader_lib: &ShaderLibrary,
    ) {
        let primitives = &self.primitives;
        let environment_textures = &assets.environment_textures;

        if let Some((environment, _transform)) = environment {
            if let Some(handle) = &environment.texture {
                if let Some(environment_texture) = environment_textures.get(handle) {
                    cmd.set_shader(shader_lib.get("skybox").unwrap());
                    cmd.set_rasterizer_state(RasterizerState {
                        cull_mode: CullMode::Front,
                        ..Default::default()
                    });
                    // cmd.set_depth_stencil_state(DepthStencilState {
                    //     depth_test_enabled: true,
                    //     depth_write_enabled: false,
                    //     depth_compare_op: CompareOp::GreaterOrEqual,
                    //     ..Default::default()
                    // });

                    let skybox = environment_texture.skybox();
                    cmd.set_buffer(&res.world_ubo, 0..1, 0, 0);

                    if environment.use_proxy {
                        cmd.set_image_mip(
                            environment_texture.specular_prefiltered(),
                            environment.mip_level,
                            0,
                            1,
                        );
                    } else {
                        cmd.set_image(skybox, 0, 1);
                    }
                    cmd.set_vertex_input_layout(self.skybox_layout);
                    cmd.set_vertex_buffers(&[&primitives.cube.verts], 0);

                    cmd.set_index_buffer(&primitives.cube.inds);

                    let ubo_data = res.world_ubo.mapped_slice()[0];

                    let mut view_transform = ubo_data.view;
                    //let mut camera_matrix: Mat4 = Mat4::from_cols_array(&camera_matrix);
                    *view_transform.col_mut(3) = Vec4::new(0.0, 0.0, 0.0, 1.0);

                    view_transform *= ubo_data.environment_transform;

                    cmd.push_constants(
                        &PushConstants {
                            view_transform,
                            material_data: MaterialPC::default(),
                        },
                        0,
                    );

                    cmd.draw_indexed(0..primitives.cube.inds.capacity(), 0, 0..1);
                }
            }
        }
    }
    fn render_world(&self, cmd: &mut RenderpassCommands, world: &World, assets: &Assets) {
        hikari_dev::profile_function!();

        let scenes = &assets.scenes;
        let textures = &assets.textures;
        let materials = &assets.materials;

        let primitives = &self.primitives;

        for (_, (transform, mesh_comp)) in &mut world.query::<(&Transform, &MeshRender)>() {
            //let mut transform = ;
            if let MeshSource::Scene(handle, mesh_ix) = &mesh_comp.source {
                if let Some(scene) = scenes.get(handle) {
                    let mesh = &scene.meshes[*mesh_ix];

                    let transform = transform.get_matrix() * mesh.transform.get_matrix();

                    for submesh in &mesh.sub_meshes {
                        {
                            hikari_dev::profile_scope!("Set vertex and index buffers");
                            cmd.set_vertex_buffers(
                                &[
                                    &submesh.position,
                                    &submesh.normals,
                                    &submesh.tc0,
                                    &submesh.tc1,
                                ],
                                0,
                            );

                            cmd.set_index_buffer(&submesh.indices);
                        }
                        let material = materials
                            .get(&submesh.material)
                            .unwrap_or_else(|| &primitives.default_mat);

                        let mut textures_present = 0;
                        textures_present |= (material.albedo.is_some() as u32) << 0;
                        textures_present |= (material.roughness.is_some() as u32) << 1;
                        textures_present |= (material.metallic.is_some() as u32) << 2;
                        textures_present |= (material.normal.is_some() as u32) << 3;
                        textures_present |= (material.emissive.is_some() as u32) << 4;
                        //let has_albedo_tex = material.albedo.is_some() as u32;
                        //let has_roughness_tex = material.roughness.is_some() as u32;
                        //let has_metallic_tex = material.metallic.is_some() as u32;
                        //let has_normal_tex = material.normal.is_some() as u32;
                        //let has_emissive_tex = material.emissive.is_some() as u32;

                        let material_data = MaterialPC {
                            albedo: material.albedo_factor,
                            roughness: material.roughness_factor,
                            metallic: material.metallic_factor,
                            emissive: material.emissive_factor * material.emissive_strength,
                            uv_set: material.uv_set,
                            textures_mask: textures_present,
                            ..Default::default()
                        };

                        let pc = PushConstants {
                            view_transform: transform,
                            material_data,
                        };

                        cmd.push_constants(&pc, 0);

                        let albedo =
                            resolve_texture(&material.albedo, &textures, &primitives.black);
                        let roughness =
                            resolve_texture(&material.roughness, &textures, &primitives.black);
                        let metallic =
                            resolve_texture(&material.metallic, &textures, &primitives.black);
                        let emissive =
                            resolve_texture(&material.emissive, &textures, &primitives.black);

                        let normal =
                            resolve_texture(&material.normal, &textures, &primitives.black);

                        cmd.set_image(albedo.raw(), 1, 0);
                        cmd.set_image(roughness.raw(), 1, 1);
                        cmd.set_image(metallic.raw(), 1, 2);
                        cmd.set_image(emissive.raw(), 1, 3);
                        cmd.set_image(normal.raw(), 1, 4);

                        cmd.draw_indexed(0..submesh.indices.capacity(), 0, 0..1);
                    }
                }
            }
        }
    }
    pub fn render(
        &mut self,
        cmd: &mut RenderpassCommands,
        world: &World,
        asset_manager: &AssetManager,
        graph_res: &GraphResources,
        res: &RenderResources,
        record_info: &PassRecordInfo,
        shader_lib: &ShaderLibrary,
    ) {
        hikari_dev::profile_function!();
        let camera = res.camera;

        if camera.is_some() {
            let assets = Assets::fetch(asset_manager);

            let mut environment_comp = world.query::<(&Environment, &Transform)>();
            let environment = environment_comp.iter().next().map(|(_, env)| env);

            cmd.set_viewport(
                0.0,
                0.0,
                record_info.framebuffer_width as f32,
                record_info.framebuffer_height as f32,
            );
            cmd.set_scissor(
                0,
                0,
                record_info.framebuffer_width,
                record_info.framebuffer_height,
            );

            if res.settings.debug.wireframe {
                cmd.set_shader(shader_lib.get("unlit").unwrap());
                cmd.set_rasterizer_state(RasterizerState {
                    polygon_mode: PolygonMode::Line,
                    line_width: 2.0,
                    ..Default::default()
                });
            } else {
                self.render_skybox(cmd, environment, &assets, res, shader_lib);
                cmd.set_shader(shader_lib.get("pbr").unwrap());
                cmd.set_rasterizer_state(RasterizerState::default());
            }
            cmd.set_depth_stencil_state(DepthStencilState {
                depth_test_enabled: true,
                depth_write_enabled: false,
                depth_compare_op: CompareOp::Equal,
                ..Default::default()
            });

            cmd.set_vertex_input_layout(self.layout);

            cmd.set_buffer(&res.world_ubo, 0..1, 0, 0);
            cmd.set_image(graph_res.get_image(&self.shadow_atlas).unwrap(), 0, 1);

            let cascade_render_buffer = graph_res.get_buffer(&self.cascade_render_buffer).unwrap();

            cmd.set_buffer(cascade_render_buffer, 0..cascade_render_buffer.len(), 0, 2);

            self.prepare_ibl(cmd, environment, &assets);
            self.render_world(cmd, world, &assets);
        } else {
            log::warn!("No camera in the world");
        }
    }
}

struct Assets<'a> {
    scenes: PoolRef<'a, Scene>,
    materials: PoolRef<'a, Material>,
    textures: PoolRef<'a, Texture2D>,
    environment_textures: PoolRef<'a, EnvironmentTexture>,
}
impl<'a> Assets<'a> {
    pub fn fetch(asset_manager: &'a AssetManager) -> Self {
        let scenes = asset_manager
            .read_assets::<Scene>()
            .expect("Meshes pool not found");
        let materials = asset_manager
            .read_assets::<Material>()
            .expect("Materials pool not found");
        let textures = asset_manager
            .read_assets::<Texture2D>()
            .expect("Textures pool not found");
        let environment_textures = asset_manager
            .read_assets::<EnvironmentTexture>()
            .expect("Environment Textures pool not found");
        Self {
            scenes,
            materials,
            textures,
            environment_textures,
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
    _device: &Arc<Device>,
    graph: &mut GraphBuilder<Args>,
    shader_lib: &mut ShaderLibrary,
    primitives: &Arc<hikari_3d::primitives::Primitives>,
    shadow_atlas: &GpuHandle<SampledImage>,
    cascade_render_buffer: &GpuHandle<GpuBuffer<CascadeRenderInfo>>,
    depth_prepass: &GpuHandle<SampledImage>,
) -> anyhow::Result<GpuHandle<SampledImage>> {
    PBRPass::build(
        graph,
        shader_lib,
        primitives,
        shadow_atlas,
        cascade_render_buffer,
        depth_prepass,
    )
}
