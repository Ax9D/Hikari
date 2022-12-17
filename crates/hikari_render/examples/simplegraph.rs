use std::sync::Arc;

use hikari_render as rg;
use simple_logger::SimpleLogger;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    window::WindowBuilder,
};

mod common;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

fn triangle_shader(device: &Arc<rg::Device>) -> Arc<rg::Shader> {
    let vertex = std::fs::read_to_string("examples/shaders/triangle.vert").unwrap();
    let fragment = std::fs::read_to_string("examples/shaders/screen.frag").unwrap();
    rg::ShaderProgramBuilder::vertex_and_fragment(
        "TriangleShader",
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(vertex),
        },
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(fragment),
        },
    )
    .build(device)
    .unwrap()
}
fn blue_shader(device: &Arc<rg::Device>) -> Arc<rg::Shader> {
    let vertex = std::fs::read_to_string("examples/shaders/screenSpaceQuad.vert").unwrap();
    let fragment = std::fs::read_to_string("examples/shaders/blue.frag").unwrap();
    rg::ShaderProgramBuilder::vertex_and_fragment(
        "BlueShader",
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(vertex),
        },
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(fragment),
        },
    )
    .build(device)
    .unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .without_timestamps()
        //.with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    hikari_dev::profiling_init();
    let window = WindowBuilder::new().with_inner_size(LogicalSize::new(WIDTH, HEIGHT));

    let (mut gfx, gameloop) = common::GameLoop::new(
        window,
        rg::GfxConfig {
            debug: true,
            features: rg::Features::default(),
            vsync: false,
            ..Default::default()
        },
    )?;

    let shader = triangle_shader(gfx.device());
    let _blue = blue_shader(gfx.device());

    let mut gb: rg::GraphBuilder<(f32, i32)> = rg::GraphBuilder::new(&mut gfx, WIDTH, HEIGHT);

    let mut frame_count = 0;
    let mut last_time = std::time::Instant::now();
    let mut state = true;

    let _blue_target = gb.create_image(
        "blue",
        rg::ImageConfig::color2d(),
        rg::ImageSize::default_xy(),
    )?;

    // gb.add_renderpass(
    //     rg::Renderpass::new("Blue", rg::ImageSize::default(), move |cmd, _, _, _| {
    //         cmd.set_shader(&blue);
    //         cmd.draw(0..6, 0..1);
    //     })
    //     .draw_image(&blue_target, rg::AttachmentConfig::color_default(1)),
    // );

    gb.add_renderpass(
        rg::Renderpass::<(f32, i32)>::new("Triangle", rg::ImageSize::default_xy())
            .cmd(move |cmd, _, record_info, (x, y)| {
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
                    record_info.framebuffer_width,
                );

                cmd.set_shader(&shader);

                let now = std::time::Instant::now();
                if now - last_time > std::time::Duration::from_secs(1) {
                    last_time = now;
                    //state = !state;
                }

                if state {
                    cmd.set_rasterizer_state(rg::RasterizerState {
                        polygon_mode: rg::PolygonMode::Fill,
                        ..Default::default()
                    });
                } else {
                    cmd.set_rasterizer_state(rg::RasterizerState {
                        polygon_mode: rg::PolygonMode::Line,
                        ..Default::default()
                    });
                }

                cmd.draw(0..3, 0..1);

                frame_count += 1;
            })
            // .sample_image(
            //     &blue_target,
            //     rg::AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
            //     3,
            // )
            .present(),
    );

    let mut graph = gb.build()?;

    gameloop.run(gfx, move |gfx, _window, event, _control_flow| {
        hikari_dev::profile_scope!("mainloop");

        match event {
            Event::MainEventsCleared => {
                graph.execute((&1.0, &2)).unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                window_id: _,
            } => {
                graph
                    .resize(size.width, size.height)
                    .expect("Failed to resize graph");
            }
            Event::LoopDestroyed => {
                graph.prepare_exit();
            }
            _ => (),
        }

        hikari_dev::finish_frame!();
    })
}
