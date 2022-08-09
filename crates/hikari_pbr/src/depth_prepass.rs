use std::sync::Arc;

use crate::Args;
use hikari_3d::*;
use hikari_math::*;
use hikari_render::*;

use crate::util::*;

#[repr(C)]
#[derive(Copy, Clone)]
struct UBO {
    view_proj: [f32; 16],
}
#[repr(C)]
#[derive(Copy, Clone)]
struct PushConstants {
    transform: hikari_math::Mat4,
}

pub fn build_pass(
    device: &Arc<Device>,
    graph: &mut GraphBuilder<Args>,
) -> anyhow::Result<GpuHandle<SampledImage>> {
    let shader = ShaderProgramBuilder::vertex_and_fragment(
        "DepthPrepass",
        &ShaderCode {
            entry_point: "main",
            data: ShaderData::Glsl(std::fs::read_to_string("assets/shaders/depth_only.vert")?),
        },
        &ShaderCode {
            entry_point: "main",
            data: ShaderData::Glsl(std::fs::read_to_string("assets/shaders/empty.frag")?),
        },
    )
    .build(device)
    .expect("Failed to create shader");

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

    let depth_output = graph
        .create_image(
            "PrepassDepth",
            ImageConfig::depth_only(device),
            ImageSize::default(),
        )
        .expect("Failed to create depth image");
    let mut ubo = PerFrame::new([
        create_uniform_buffer::<UBO>(device, 1)?,
        create_uniform_buffer::<UBO>(device, 1)?,
    ]);

    graph.add_renderpass(
        Renderpass::<Args>::new(
            "DepthPrepass",
            ImageSize::default(),
            move |cmd, (world, config, assets)| {
                cmd.set_shader(&shader);
                cmd.set_vertex_input_layout(layout);

                cmd.set_depth_stencil_state(DepthStencilState {
                    depth_test_enabled: true,
                    depth_write_enabled: true,
                    depth_compare_op: CompareOp::Less,
                    ..Default::default()
                });
                let camera = get_camera(world);

                if let Some(camera_entity) = camera {
                    let camera = world.get_component::<&Camera>(camera_entity).unwrap();
                    let camera_transform = world.get_component::<&Transform>(camera_entity).unwrap();

                    let proj = camera.get_projection_matrix(config.viewport.0, config.viewport.1);
                    let view = camera_transform.get_matrix().inverse();
                    let view_proj = (proj * view).to_cols_array();

                    ubo.get_mut().mapped_slice_mut()[0] = UBO { view_proj };

                    cmd.set_uniform_buffer(ubo.get(), 0..1, 0, 0);

                    let scenes = assets.get::<Scene>().expect("Scenes pool not found");
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

                                        cmd.push_constants(&PushConstants { transform }, 0);

                                        // println!(
                                        //     "{:?} {:?} {:?} {:?}",
                                        //     albedo.raw().image(),
                                        //     roughness.raw().image(),
                                        //     metallic.raw().image(),
                                        //     normal.raw().image()
                                        // );

                                        cmd.draw_indexed(0..submesh.indices.capacity(), 0, 0..1);
                                    }
                                }
                            }
                            MeshSource::None => {}
                        }
                    }
                }
                ubo.next_frame();
            },
        )
        .draw_image(&depth_output, AttachmentConfig::depth_only_default()),
    );

    Ok(depth_output)
}
