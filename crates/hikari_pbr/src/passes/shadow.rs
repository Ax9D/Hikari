use std::sync::Arc;

use crate::{light::CascadeRenderInfo, world::WorldUBO, Args, Settings};
use hikari_3d::*;
use hikari_math::*;
use hikari_render::*;

pub const N_CASCADES: usize = 4;
//pub const SHADOW_MAP_SIZE: u32 = 1024;

#[repr(C)]
#[derive(Copy, Clone)]
struct PushConstants {
    transform: hikari_math::Mat4,
    cascade_ix: u32,
}

pub fn compute_cascades(world_ubo: &mut WorldUBO, settings: &Settings) {
    let shadow_map_size = settings.directional_shadow_map_resolution.size();

    for i in 0..N_CASCADES {
        let atlas_width = N_CASCADES as u32 * shadow_map_size;
        let atlas_height = shadow_map_size;

        let atlas_size_ratio = Vec2::new(
            shadow_map_size as f32 / atlas_width as f32,
            shadow_map_size as f32 / atlas_height as f32,
        );
        let atlas_uv_offset = Vec2::new(i as f32 * atlas_size_ratio.x, 0.0);

        let map_texel_size = 1.0 / shadow_map_size as f32;

        world_ubo.dir_light.cascades[i].map_size = shadow_map_size as f32;
        world_ubo.dir_light.cascades[i].map_texel_size = map_texel_size;
        world_ubo.dir_light.cascades[i].atlas_size_ratio = atlas_size_ratio;
        world_ubo.dir_light.cascades[i].atlas_uv_offset = atlas_uv_offset;
    }
}
const NUM_THREADS: u32 = 16;
pub fn create_hi_z_images(
    device: &Arc<Device>,
    mut width: u32,
    mut height: u32,
) -> anyhow::Result<Vec<SampledImage>> {
    let mut hiz_images = Vec::new();

    let mut config = ImageConfig {
        format: vk::Format::R32G32_SFLOAT,
        filtering: vk::Filter::NEAREST,
        wrap_x: vk::SamplerAddressMode::CLAMP_TO_BORDER,
        wrap_y: vk::SamplerAddressMode::CLAMP_TO_BORDER,
        wrap_z: vk::SamplerAddressMode::CLAMP_TO_BORDER,
        sampler_reduction_mode: None,
        aniso_level: 0.0,
        mip_levels: 1,
        mip_filtering: vk::SamplerMipmapMode::NEAREST,
        usage: vk::ImageUsageFlags::STORAGE,
        flags: vk::ImageCreateFlags::empty(),
        image_type: vk::ImageType::TYPE_2D,
        image_view_type: vk::ImageViewType::TYPE_2D,
        initial_layout: vk::ImageLayout::GENERAL,
        host_readable: false,
    };

    while width > 1 || height > 1 {
        width = n_workgroups(width, NUM_THREADS);
        height = n_workgroups(height, NUM_THREADS);

        // Make last mip host readable for readback
        if width == 1 && height == 1 {
            config.host_readable = true;
        }

        let hiz_image = SampledImage::with_dimensions(device, width, height, 1, 1, config)?;
        hiz_images.push(hiz_image);
    }

    Ok(hiz_images)
}
pub fn build_pass(
    device: &Arc<Device>,
    graph: &mut GraphBuilder<Args>,
    shader_lib: &mut ShaderLibrary,
    settings: &Settings,
    depth_prepass: &GpuHandle<SampledImage>,
) -> anyhow::Result<(
    GpuHandle<SampledImage>,
    GpuHandle<GpuBuffer<CascadeRenderInfo>>,
)> {
    let shadow_map_size = settings.directional_shadow_map_resolution.size();

    let atlas_size = ImageSize::absolute_xy(shadow_map_size * N_CASCADES as u32, shadow_map_size);
    let mut config = ImageConfig::depth_only_attachment(device);
    config.format = vk::Format::D32_SFLOAT;
    let shadow_atlas = graph.create_image("ShadowMapAtlas", config, atlas_size)?;

    // let mandelbrot = graph.create_image("Mandelbrot", config, ImageSize::absolute_xy(1024, 1024))?;

    // graph.add_computepass(ComputePass::<Args>::new("ResetLayouts")
    // .read_image(&mandelbrot, AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer)
    // );

    // graph.add_computepass(ComputePass::<Args>::new("MandelbrotCompute")
    // .write_image(&mandelbrot, AccessType::ComputeShaderWrite)
    // .cmd(move|cmd, graph_res, _, (_, _, shader_lib, _)| {
    //     cmd.set_image(graph_res.get_image(&mandelbrot).unwrap(), 0, 0);
    //     cmd.set_shader(shader_lib.get("mandelbrot").unwrap());
    //     cmd.dispatch((1024/8, 1024/8, 1));
    // }));

    let layout = VertexInputLayout::builder()
        .buffer(&[ShaderDataType::Vec3f], StepMode::Vertex)
        .build();

    shader_lib.insert("depth_reduce_initial")?;
    shader_lib.insert("depth_reduce")?;

    let depth_prepass = depth_prepass.clone();
    graph.add_computepass(
        ComputePass::<Args>::new("HierarchicalDepthGeneration")
            .read_image(
                &depth_prepass,
                AccessType::ComputeShaderReadSampledImageOrUniformTexelBuffer,
            )
            .cmd(
                move |cmd, graph_res, _record_info, (_world, res, shader_lib, _assets)| {
                    let hiz_images = &res.hi_z_images;

                    let depth_output = graph_res.get_image(&depth_prepass).unwrap();
                    let depth_only_view = depth_output.custom_image_view(ImageViewDesc {
                        view_type: vk::ImageViewType::TYPE_2D,
                        aspect: vk::ImageAspectFlags::DEPTH,
                        mip_range: 0..1,
                        layer_range: 0..1,
                    });

                    cmd.set_shader(shader_lib.get("depth_reduce_initial").unwrap());

                    cmd.set_buffer(&res.world_ubo, 0..1, 0, 0);
                    cmd.set_image_view_and_sampler(
                        depth_only_view,
                        depth_output.sampler(),
                        0,
                        1,
                        0,
                    );
                    cmd.set_image(&hiz_images[0], 0, 2);

                    cmd.dispatch((hiz_images[0].width(), hiz_images[0].height(), 1));

                    cmd.set_shader(shader_lib.get("depth_reduce").unwrap());

                    for i in 1..hiz_images.len() {
                        cmd.apply_image_barrier(
                            &hiz_images[i - 1],
                            &[AccessType::ComputeShaderWrite],
                            &[AccessType::ComputeShaderReadOther],
                            vk_sync::ImageLayout::General,
                            vk_sync::ImageLayout::General,
                            vk::ImageSubresourceRange {
                                aspect_mask: vk::ImageAspectFlags::COLOR,
                                base_mip_level: 0,
                                level_count: 1,
                                base_array_layer: 0,
                                layer_count: 1,
                            },
                        );

                        cmd.set_image(&hiz_images[i - 1], 0, 1);
                        cmd.set_image(&hiz_images[i], 0, 2);
                        cmd.dispatch((hiz_images[i].width(), hiz_images[i].height(), 1));
                    }
                },
            ),
    );
    let cascade_render_buffer = GpuBuffer::<CascadeRenderInfo>::new(
        device,
        N_CASCADES,
        vk::BufferUsageFlags::STORAGE_BUFFER,
    )?;
    let cascade_render_buffer = graph.add_buffer("CascadeRenderInfo".into(), cascade_render_buffer);

    shader_lib.insert("generate_cascades")?;

    graph.add_computepass(
        ComputePass::<Args>::new("ShadowCascadeGeneration")
            .write_buffer(&cascade_render_buffer, AccessType::ComputeShaderWrite)
            .cmd(
                move |cmd, graph_res, _, (world, res, shader_lib, _assets)| {
                    if let Some(dir_light) = res.directional_light {
                        let light = world.get_component::<&Light>(dir_light).unwrap();
                        if !light.shadow.enabled {
                            return;
                        }

                        let reduced_depth_image = res.hi_z_images.last().unwrap();
                        let cascade_render_buffer =
                            graph_res.get_buffer(&cascade_render_buffer).unwrap();

                        cmd.set_shader(shader_lib.get("generate_cascades").unwrap());

                        cmd.set_buffer(&res.world_ubo, 0..1, 0, 0);
                        cmd.set_image(reduced_depth_image, 0, 1);
                        cmd.set_buffer(cascade_render_buffer, 0..cascade_render_buffer.len(), 0, 2);

                        //FIXME: Do this automatically; Implement graph external resources
                        cmd.apply_image_barrier(
                            &reduced_depth_image,
                            &[AccessType::ComputeShaderWrite],
                            &[AccessType::ComputeShaderReadOther],
                            vk_sync::ImageLayout::General,
                            vk_sync::ImageLayout::General,
                            vk::ImageSubresourceRange {
                                aspect_mask: vk::ImageAspectFlags::COLOR,
                                base_mip_level: 0,
                                level_count: 1,
                                base_array_layer: 0,
                                layer_count: 1,
                            },
                        );

                        cmd.dispatch((1, 1, 1));
                    }
                },
            ),
    );

    shader_lib.insert("shadow")?;
    graph.add_renderpass(
        Renderpass::<Args>::new("ShadowMapping", atlas_size)
            .read_buffer(&cascade_render_buffer, AccessType::VertexShaderReadOther)
            .draw_image(&shadow_atlas, AttachmentConfig::depth_only_default())
            .cmd(move |cmd, graph_res, _, (world, res, shader_lib, assets)| {
                let dir_light = res.directional_light;

                if let Some(dir_light) = dir_light {
                    cmd.set_shader(shader_lib.get("shadow").unwrap());

                    let light = world.get_component::<&Light>(dir_light).unwrap();
                    if !light.shadow.enabled {
                        return;
                    }

                    let shadow_info = &light.shadow;

                    cmd.set_vertex_input_layout(layout);

                    cmd.set_depth_stencil_state(DepthStencilState {
                        depth_test_enabled: true,
                        depth_write_enabled: true,
                        depth_compare_op: CompareOp::LessOrEqual,
                        ..Default::default()
                    });

                    cmd.set_rasterizer_state(RasterizerState {
                        cull_mode: if shadow_info.cull_front_face {
                            CullMode::Front
                        } else {
                            CullMode::Back
                        },
                        depth_bias_enable: true,
                        depth_bias_slope_factor: shadow_info.slope_scaled_bias,
                        depth_clamp_enable: true,
                        ..Default::default()
                    });

                    cmd.set_buffer(&res.world_ubo, 0..1, 0, 0);

                    let cascade_render_buffer =
                        graph_res.get_buffer(&cascade_render_buffer).unwrap();
                    cmd.set_buffer(cascade_render_buffer, 0..N_CASCADES, 0, 1);

                    let scenes = assets
                        .read_assets::<Scene>()
                        .expect("Scenes pool not found");
                    for cascade_ix in 0..N_CASCADES {
                        cmd.set_viewport(
                            (cascade_ix as u32 * shadow_map_size) as f32,
                            0.0,
                            shadow_map_size as f32,
                            shadow_map_size as f32,
                        );
                        cmd.set_scissor(
                            (cascade_ix as u32 * shadow_map_size) as i32,
                            0,
                            shadow_map_size,
                            shadow_map_size,
                        );

                        for (_, (transform, mesh_comp)) in
                            &mut world.query::<(&Transform, &MeshRender)>()
                        {
                            let mut transform = transform.get_matrix();
                            if let MeshSource::Scene(handle, mesh_ix) = &mesh_comp.source {
                                if let Some(scene) = scenes.get(handle) {
                                    let mesh = &scene.meshes[*mesh_ix];

                                    transform *= mesh.transform.get_matrix();

                                    cmd.push_constants(
                                        &PushConstants {
                                            transform,
                                            cascade_ix: cascade_ix as u32,
                                        },
                                        0,
                                    );

                                    for submesh in &mesh.sub_meshes {
                                        {
                                            hikari_dev::profile_scope!(
                                                "Set vertex and index buffers"
                                            );
                                            cmd.set_vertex_buffer(&submesh.position, 0);
                                            cmd.set_index_buffer(&submesh.indices);
                                        }

                                        cmd.draw_indexed(0..submesh.indices.capacity(), 0, 0..1);
                                    }
                                }
                            }
                        }
                    }
                }
            }),
    );

    Ok((shadow_atlas, cascade_render_buffer))
}
