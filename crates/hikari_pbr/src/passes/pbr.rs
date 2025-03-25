use std::sync::Arc;

use hikari_3d::{primitives::Primitives, *};
use hikari_asset::{AssetManager, AssetPool, PoolRef};
use hikari_math::*;
use hikari_render::{*,};

use crate::{light::CascadeRenderInfo, resources::RenderResources, Args, Settings, DebugView, common::{MaterialInputs, PushConstants}, instancing::MeshInstancer};

struct PBRPass {
    layout: VertexInputLayout,
    skybox_layout: VertexInputLayout,
    primitives: Arc<Primitives>,
    shadow_atlas: GpuHandle<SampledImage>,
    cascade_render_buffer: GpuHandle<GpuBuffer<CascadeRenderInfo>>,
    shader_ids: ShaderIds
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

        let lit = shader_lib.insert_with_defines("pbr", &["LIGHT_MODE_LIT"])?;
        let unlit = shader_lib.insert_with_defines("pbr", &["LIGHT_MODE_UNLIT"])?;
        let skybox = shader_lib.insert("skybox")?;
        let outline = shader_lib.insert("outline")?;

        let shader_ids = ShaderIds {
            lit,
            unlit,
            skybox,
            outline,
        };

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
            shader_ids
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
                    kind: AttachmentKind::DepthStencil,
                    access: AccessType::StencilAttachmentWriteDepthReadOnly,
                    load_op: hikari_render::vk::AttachmentLoadOp::LOAD,
                    store_op: hikari_render::vk::AttachmentStoreOp::STORE,
                    stencil_load_op: hikari_render::vk::AttachmentLoadOp::LOAD,
                    stencil_store_op: hikari_render::vk::AttachmentStoreOp::STORE,
                },
            )
            .cmd(
                move |cmd, graph_res, record_info, (_world, res, shader_lib, asset_manager)| {
                    renderer.render(
                        cmd,
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
    fn render_skybox(
        &self,
        cmd: &mut RenderpassCommands,
        assets: &Assets,
        res: &RenderResources,
    ) {
        cmd.begin_debug_region("Skybox", Vec4::new(0.46,0.69,0.96,1.0));
        let primitives = &self.primitives;

        cmd.set_shader(assets.skybox_shader);
        cmd.set_rasterizer_state(RasterizerState {
            cull_mode: CullMode::Front,
            ..Default::default()
        });

        cmd.set_vertex_input_layout(self.skybox_layout);
        cmd.set_vertex_buffers(&[&primitives.cube.verts], 0);

        cmd.set_index_buffer(&primitives.cube.inds);

        let ubo_data = res.world_ubo.mapped_slice()[0];

        let mut view_transform = ubo_data.view;

        *view_transform.col_mut(3) = Vec4::new(0.0, 0.0, 0.0, 1.0);

        view_transform *= ubo_data.environment_transform;

        cmd.push_constants(
            &PushConstants {
                transform: view_transform,
                mat: MaterialInputs::default(),
            },
            0,
        );

        cmd.draw_indexed(0..primitives.cube.inds.capacity(), 0, 0..1);

        cmd.end_debug_region();
    }
    fn render_world(&self, cmd: &mut RenderpassCommands, instancer: &MeshInstancer, assets: &Assets, settings: &Settings) {
        hikari_dev::profile_function!();
        cmd.begin_debug_region("Draw Static Meshes", Vec4::new(0.33,0.25,0.75, 1.0));

        cmd.set_depth_stencil_state(DepthStencilState {
            depth_test_enabled: true,
            depth_write_enabled: false,
            depth_compare_op: CompareOp::Equal,
            ..Default::default()
        });

        for (instance_id, batch) in instancer.batches() {
            self.draw_sub_mesh(cmd, instance_id, batch.count(), batch.submesh(), assets, settings);
        }
        // cmd.set_depth_stencil_state(DepthStencilState {
        //     depth_test_enabled: true,
        //     depth_write_enabled: false,
        //     stencil_test_enabled: true,
        //     stencil_test_write_mask: 0xFF,
        //     stencil_test_compare_mask: 0xFF,
        //     stencil_test_reference: 0xFF,
        //     stencil_test_compare_op: CompareOp::Always,
        //     stencil_test_pass_op: StencilOp::Replace,
        //     stencil_test_fail_op: StencilOp::Replace,
        //     stencil_test_depth_fail_op: StencilOp::Replace,
        //     depth_compare_op: CompareOp::Equal,
        //     ..Default::default()
        // });

        // for (_, (transform, mesh_comp, _)) in
        //     &mut world.query::<(&MeshRender, &Outline)>()
        // {
        //     let Some(mesh) = mesh_comp.get_mesh(scenes) else {continue};
        //     let transform = transform.get_matrix() * mesh.transform.get_matrix();

        //     for submesh in &mesh.sub_meshes {
        //         self.draw_sub_mesh(cmd, transform, submesh, assets, settings);
        //     }
        // }

        // cmd.set_shader(assets.outline_shader);

        // cmd.set_depth_stencil_state(DepthStencilState {
        //     depth_test_enabled: false,
        //     depth_write_enabled: false,
        //     stencil_test_enabled: true,
        //     stencil_test_write_mask: 0xFF,
        //     stencil_test_compare_mask: 0xFF,
        //     stencil_test_reference: 0xFF,
        //     stencil_test_compare_op: CompareOp::NotEqual,
        //     stencil_test_pass_op: StencilOp::Replace,
        //     stencil_test_fail_op: StencilOp::Keep,
        //     stencil_test_depth_fail_op: StencilOp::Keep,
        //     ..Default::default()
        // });

        // for (_, (transform, mesh_comp, outline)) in
        //     &mut world.query::<(&Transform, &MeshRender, &Outline)>()
        // {
        //     let Some(mesh) = mesh_comp.get_mesh(scenes) else {continue};
        //     let transform = transform.get_matrix() * mesh.transform.get_matrix();

        //     for submesh in &mesh.sub_meshes {
        //         self.draw_sub_mesh_outline(cmd, outline, transform, submesh);
        //     }
        // }

        cmd.end_debug_region();
    }
    fn draw_sub_mesh(
        &self,
        cmd: &mut RenderpassCommands,
        instance_id: usize,
        instance_count: usize,
        submesh: &SubMesh,
        assets: &Assets,
        settings: &Settings,
    ) {
        let primitives = &self.primitives;
        let textures = &assets.textures;
        let materials = &assets.materials;
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

        // let mut textures_mask = 0;
        // textures_mask |= (material.albedo.is_some() as u32) << 0;
        // textures_mask |= (material.roughness.is_some() as u32) << 1;
        // textures_mask |= (material.metallic.is_some() as u32) << 2;
        // textures_mask |= (material.normal.is_some() as u32) << 3;
        // textures_mask |= (material.emissive.is_some() as u32) << 4;
        //let has_albedo_tex = material.albedo.is_some() as u32;
        //let has_roughness_tex = material.roughness.is_some() as u32;
        //let has_metallic_tex = material.metallic.is_some() as u32;
        //let has_normal_tex = material.normal.is_some() as u32;
        //let has_emissive_tex = material.emissive.is_some() as u32;

        let mut albedo_ix = resolve_texture_bindless(&material.albedo, &textures, &primitives.checkerboard);
        let roughness_ix = resolve_texture_bindless(&material.roughness, &textures, &primitives.black);
        let metallic_ix = resolve_texture_bindless(&material.metallic, &textures, &primitives.black);
        let emissive_ix = resolve_texture_bindless(&material.emissive, &textures, &primitives.black);
        let normal_ix = resolve_texture_bindless(&material.normal, &textures, &primitives.black);

        let albedo;
        if settings.debug.view == DebugView::Wireframe {
            albedo_ix = -1;
            albedo = Vec4::new(0.0, 0.5, 0.6, 1.0); // Blue
        } else {
            albedo = material.albedo_factor;
        }

        let mat = MaterialInputs {
            albedo,
            roughness: material.roughness_factor,
            metallic: material.metallic_factor,
            emissive: material.emissive_factor * material.emissive_strength,
            uv_set: material.uv_set,
            albedo_ix,
            roughness_ix,
            metallic_ix,
            emissive_ix,
            normal_ix,
            ..Default::default()
        };

        // let pc = PushConstants {
        //     mat,
        //     ..Default::default()
        // };

        cmd.push_constants(&mat, std::mem::size_of::<Mat4>());
        
        // cmd.set_image(albedo.raw(), 3, 0);
        // cmd.set_image(roughness.raw(), 3, 1);
        // cmd.set_image(metallic.raw(), 3, 2);
        // cmd.set_image(emissive.raw(), 3, 3);
        // cmd.set_image(normal.raw(), 3, 4);

        cmd.draw_indexed(0..submesh.indices.capacity(), 0, instance_id..instance_id + instance_count);
    }
    fn draw_sub_mesh_outline(
        &self,
        cmd: &mut RenderpassCommands,
        outline: &Outline,
        instance_id: usize,
        submesh: &SubMesh,
    ) {
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
        let mat = MaterialInputs {
            albedo: Vec4::from((outline.color, outline.thickness)),
            ..Default::default()
        };

        let pc = PushConstants {
            mat,
            ..Default::default()    
        };

        cmd.push_constants(&pc, 0);

        cmd.draw_indexed(0..submesh.indices.capacity(), 0, instance_id..instance_id + 1);
    }
    pub fn render(
        &mut self,
        cmd: &mut RenderpassCommands,
        asset_manager: &AssetManager,
        graph_res: &GraphResources,
        res: &RenderResources,
        record_info: &PassRecordInfo,
        shader_lib: &ShaderLibrary,
    ) {
        hikari_dev::profile_function!();
        let camera = res.camera;

        if camera.is_some() {
            let assets = Assets::fetch(asset_manager, shader_lib, &self.shader_ids);
            
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

            match res.settings.debug.view {
                DebugView::Wireframe => {
                    cmd.set_shader(assets.unlit_shader);
                    cmd.set_rasterizer_state(RasterizerState {
                        polygon_mode: PolygonMode::Line,
                        line_width: 2.0,
                        ..Default::default()
                    });
                },
                DebugView::Unlit => {
                    self.render_skybox(cmd, &assets, res);
                    cmd.set_shader(assets.unlit_shader);
                    cmd.set_rasterizer_state(RasterizerState::default());
                },
                DebugView::None => {
                    self.render_skybox(cmd, &assets, res);
                    cmd.set_shader(assets.pbr_shader);
                    cmd.set_rasterizer_state(RasterizerState::default());
                }
            }

            cmd.set_vertex_input_layout(self.layout);
            
            let cascade_render_buffer = graph_res.get_buffer(&self.cascade_render_buffer).unwrap();
            let shadow_atlas = graph_res.get_image(&self.shadow_atlas).unwrap();
            
            cmd.set_buffer(cascade_render_buffer, 0..cascade_render_buffer.len(), 2, 0);
            cmd.set_image(shadow_atlas, 2, 1);
            
            self.render_world(cmd, &res.mesh_instancer, &assets, &res.settings);
        } else {
            log::warn!("No camera in the world");
        }
    }
}

struct ShaderIds {
    lit: ShaderId,
    unlit: ShaderId,
    outline: ShaderId,
    skybox: ShaderId,
}
struct Assets<'a> {
    materials: PoolRef<'a, Material>,
    textures: PoolRef<'a, Texture2D>,

    pbr_shader: &'a Arc<Shader>,
    unlit_shader: &'a Arc<Shader>,
    outline_shader: &'a Arc<Shader>,
    skybox_shader: &'a Arc<Shader>,
}
impl<'a> Assets<'a> {
    pub fn fetch(asset_manager: &'a AssetManager, shader_lib: &'a ShaderLibrary, shader_ids: &ShaderIds) -> Self {
        // let scenes = asset_manager
        //     .read_assets::<Scene>()
        //     .expect("Meshes pool not found");
        let materials = asset_manager
            .read_assets::<Material>()
            .expect("Materials pool not found");
        let textures = asset_manager
            .read_assets::<Texture2D>()
            .expect("Textures pool not found");

        let pbr_shader = shader_lib.get_by_id(shader_ids.lit).expect("Failed to fetch PBR Shader");
        let unlit_shader = shader_lib.get_by_id(shader_ids.unlit).expect("Failed to fetch unlit Shader");
        let outline_shader = shader_lib
            .get_by_id(shader_ids.outline)
            .expect("Failed to get outline shader");
        let skybox_shader = shader_lib.get_by_id(shader_ids.skybox).expect("Failed to get skybox shader");

        Self {
            materials,
            textures,
            pbr_shader, 
            unlit_shader,
            outline_shader,
            skybox_shader
        }
    }
}

// fn resolve_texture<'a>(
//     handle: &Option<hikari_asset::Handle<Texture2D>>,
//     textures: &'a AssetPool<Texture2D>,
//     default: &'a Texture2D,
// ) -> &'a Texture2D {
//     handle
//         .as_ref()
//         .map(|handle| textures.get(handle).unwrap_or(default))
//         .unwrap_or(default)
// }
fn resolve_texture_bindless<'a>(
    handle: &Option<hikari_asset::Handle<Texture2D>>,
    textures: &'a AssetPool<Texture2D>,
    default: &'a Texture2D,
) -> i32 {
    hikari_dev::profile_function!();
    let index = handle
        .as_ref()
        .map(|handle| {
            let texture = textures.get(handle).unwrap_or(default);
            let bindless_handle = texture.raw().bindless_handle(0);
            bindless_handle.index() as i32
        })
        .unwrap_or(-1);

    index
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
