use std::sync::Arc;

use crate::{Args, DebugView};
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
        .buffer(&[ShaderDataType::Vec3f], StepMode::Vertex)
        .build();

    let depth_output = graph
        .create_image(
            "PrepassDepth",
            ImageConfig::depth_stencil_attachment(device),
            ImageSize::default_xy(),
        )
        .expect("Failed to create depth image");

    graph.add_renderpass(
        Renderpass::<Args>::new("DepthPrepass", ImageSize::default_xy())
            .draw_image(&depth_output, AttachmentConfig::depth_stencil_default())
            .cmd(
                move |cmd, _, record_info, (_world, res, shader_lib, _assets)| {
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

                    cmd.set_shader(shader_lib.get("depth_only").unwrap());
                    cmd.set_vertex_input_layout(layout);

                    if res.settings.debug.view == DebugView::Wireframe {
                        cmd.set_rasterizer_state(RasterizerState {
                            polygon_mode: PolygonMode::Line,
                            line_width: 2.0,
                            ..Default::default()
                        });
                    }

                    cmd.set_depth_stencil_state(DepthStencilState {
                        depth_test_enabled: true,
                        depth_write_enabled: true,
                        depth_compare_op: CompareOp::LessOrEqual,
                        ..Default::default()
                    });
                    let camera = res.camera;

                    if camera.is_some() {
                        let instancer = &res.mesh_instancer;
                        for (instance_id, batch) in instancer.batches() {
                            let submesh = batch.submesh();
                            {
                                hikari_dev::profile_scope!("Set vertex and index buffers");
                                cmd.set_vertex_buffer(&submesh.position, 0);
                                cmd.set_index_buffer(&submesh.indices);
                            }

                            cmd.draw_indexed(0..submesh.indices.capacity(), 0, instance_id..instance_id+batch.count());
                        }
                    }
                },
            ),
    );

    Ok(depth_output)
}
