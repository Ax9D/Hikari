use crate::Args;
use std::sync::Arc;

use hikari_3d::ShaderLibrary;
use hikari_math::*;
use hikari_render::*;

#[repr(C)]
#[derive(Copy, Clone)]
struct PushConstants {
    res: hikari_math::Vec2,
    enabled: i32,
}

#[cfg(feature = "editor")]
pub fn build_pass(
    _device: &Arc<Device>,
    graph: &mut GraphBuilder<Args>,
    shader_lib: &mut ShaderLibrary,
    pbr_output: &GpuHandle<SampledImage>,
) -> anyhow::Result<GpuHandle<SampledImage>> {
    shader_lib.insert("fxaa")?;
    let output = graph
        .create_image(
            "FXAAOutput",
            ImageConfig::color2d(),
            ImageSize::default_xy(),
        )
        .expect("Failed to create fxaa output");

    let pbr_output = pbr_output.clone();
    graph.add_renderpass(
        Renderpass::<Args>::new("FXAA", ImageSize::default_xy())
            .draw_image(&output, AttachmentConfig::color_default(0))
            .read_image(
                &pbr_output,
                AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
            )
            .cmd(
                move |cmd, graph_res, record_info, (_, res, shader_lib, _)| {
                    cmd.set_image(graph_res.get_image(&pbr_output).unwrap(), 0, 0);

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
                    cmd.set_rasterizer_state(RasterizerState {
                        cull_mode: CullMode::Back,
                        ..Default::default()
                    });
                    cmd.set_shader(shader_lib.get("fxaa").unwrap());

                    cmd.push_constants(
                        &PushConstants {
                            res: hikari_math::vec2(res.viewport.0 as f32, res.viewport.1 as f32),
                            enabled: res.settings.fxaa as _,
                        },
                        0,
                    );

                    cmd.draw(0..6, 0..1);
                },
            ),
    );

    Ok(output)
}

#[cfg(not(feature = "editor"))]
pub fn build_pass(
    _device: &Arc<Device>,
    graph: &mut GraphBuilder<Args>,
    shader_lib: &mut ShaderLibrary,
    pbr_output: &GpuHandle<SampledImage>,
) -> anyhow::Result<()> {
    shader_lib.insert("fxaa")?;
    let output = graph
        .create_image(
            "FXAAOutput",
            ImageConfig::color2d(),
            ImageSize::default_xy(),
        )
        .expect("Failed to create fxaa output");

    let pbr_output = pbr_output.clone();

    graph.add_renderpass(
        Renderpass::<Args>::new("FXAA", ImageSize::default_xy())
            .read_image(
                &pbr_output,
                AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
            )
            .present()
            .cmd(
                move |cmd, graph_res, record_info, (_, res, shader_lib, _)| {
                    cmd.set_image(graph_res.get_image(&pbr_output).unwrap(), 0, 0);

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

                    cmd.set_shader(shader_lib.get("fxaa").unwrap());

                    cmd.push_constants(
                        &PushConstants {
                            res: hikari_math::vec2(res.viewport.0 as f32, res.viewport.1 as f32),
                            enabled: res.settings.fxaa as _,
                        },
                        0,
                    );

                    cmd.draw(0..6, 0..1);
                },
            ),
    );

    Ok(())
}
