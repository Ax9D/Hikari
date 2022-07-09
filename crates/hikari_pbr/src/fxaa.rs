use crate::Args;
use std::sync::Arc;

use hikari_math::*;
use hikari_render::*;

#[repr(C)]
#[derive(Copy, Clone)]
struct PushConstants {
    res: hikari_math::Vec2,
    enabled: i32,
}

pub fn build_pass(
    device: &Arc<Device>,
    graph: &mut GraphBuilder<Args>,
    pbr_output: &GpuHandle<SampledImage>,
) -> anyhow::Result<GpuHandle<SampledImage>> {
    let vertex = r"
    #version 450
    vec2 positions[6] = vec2[](
        vec2(-1, -1),
        vec2(1, -1),
        vec2(1, 1),
        vec2(-1, 1),
        vec2(-1, -1),
        vec2(1, 1)
    );
    vec2 texCoords[6] = vec2[](
        vec2(0, 1),
        vec2(1, 1),
        vec2(1, 0),
        vec2(0, 0),
        vec2(0, 1),
        vec2(1, 0)
    );
    layout(location = 0) out vec2 texCoord;
    void main() {
        gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
        texCoord = texCoords[gl_VertexIndex];
    }
    ";

    let shader = ShaderProgramBuilder::vertex_and_fragment(
        "FXAA",
        &ShaderCode {
            entry_point: "main",
            data: ShaderData::Glsl(vertex.to_string()),
        },
        &ShaderCode {
            entry_point: "main",
            data: ShaderData::Glsl(std::fs::read_to_string("assets/shaders/fxaa.frag")?),
        },
    )
    .build(device)
    .expect("Failed to create shader");

    let output = graph
        .create_image("FXAAOutput", ImageConfig::color2d(), ImageSize::default())
        .expect("Failed to create fxaa output");
    
    graph.add_renderpass(
        Renderpass::<Args>::new(
            "FXAA",
            ImageSize::default(),
            move |cmd, (_, config, _)| {
                cmd.set_shader(&shader);

                cmd.push_constants(
                    &PushConstants {
                        res: hikari_math::vec2(config.width as f32, config.height as f32),
                        enabled: config.settings.fxaa as _,
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
