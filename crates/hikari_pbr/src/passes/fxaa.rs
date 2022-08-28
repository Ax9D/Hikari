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

pub fn build_pass(
    _device: &Arc<Device>,
    graph: &mut GraphBuilder<Args>,
    shader_lib: &mut ShaderLibrary,
    pbr_output: &GpuHandle<SampledImage>,
) -> anyhow::Result<GpuHandle<SampledImage>> {
    shader_lib.insert("fxaa")?;

    let output = graph
        .create_image("FXAAOutput", ImageConfig::color2d(), ImageSize::default_xy())
        .expect("Failed to create fxaa output");

    graph.add_renderpass(
        Renderpass::<Args>::new(
            "FXAA",
            ImageSize::default_xy(),
            move |cmd, (_, res, shader_lib, _)| {
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
        )
        .draw_image(&output, AttachmentConfig::color_default(0))
        .sample_image(
            &pbr_output,
            AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
            0,
        ),
    );

    Ok(output)
}
