use std::sync::Arc;

use crate::Args;
use hikari_3d::*;
use hikari_math::*;
use hikari_render::*;

#[repr(C)]
#[derive(Copy, Clone)]
struct PushConstants {
    transform: hikari_math::Mat4,
}

pub fn build_pass(
    device: &Arc<Device>,
    graph: &mut GraphBuilder<Args>,
    shader_lib: &mut ShaderLibrary,
) -> anyhow::Result<GpuHandle<SampledImage>> {

    shader_lib.insert("depth_only")?;

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
            ImageSize::default_xy(),
        )
        .expect("Failed to create depth image");

    graph.add_renderpass(
        Renderpass::<Args>::new(
            "DepthPrepass",
            ImageSize::default_xy(),
            move |cmd, (world, res, shader_lib, assets)| {
                cmd.set_shader(shader_lib.get("depth_only").unwrap());
                cmd.set_vertex_input_layout(layout);

                cmd.set_depth_stencil_state(DepthStencilState {
                    depth_test_enabled: true,
                    depth_write_enabled: true,
                    depth_compare_op: CompareOp::Less,
                    ..Default::default()
                });
                let camera = res.camera;

                if camera.is_some() {

                    cmd.set_uniform_buffer(res.world_ubo.get(), 0..1, 0, 0);

                    let scenes = assets.get::<Scene>().expect("Scenes pool not found");
                    for (_, (transform, mesh_comp)) in
                        &mut world.query::<(&Transform, &MeshRender)>()
                    {
                        let mut transform = transform.get_matrix();
                        match &mesh_comp.source {
                            MeshSource::Scene(handle, mesh_ix) => {
                                if let Some(scene) = scenes.get(handle) {
                                    let mesh = &scene.meshes[*mesh_ix];

                                    transform *= mesh.transform.get_matrix();

                                    cmd.push_constants(&PushConstants { transform }, 0);

                                    for submesh in &mesh.sub_meshes {
                                        {
                                            hikari_dev::profile_scope!(
                                                "Set vertex and index buffers"
                                            );
                                            cmd.set_vertex_buffer(&submesh.vertices, 0);
                                            cmd.set_index_buffer(&submesh.indices);
                                        }

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
            },
        )
        .draw_image(&depth_output, AttachmentConfig::depth_only_default()),
    );

    Ok(depth_output)
}
