use std::sync::Arc;

use simple_logger::SimpleLogger;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use graphy as rg;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

fn triangle_shader(device: &Arc<rg::Device>) -> Arc<rg::Shader> {
    let vertex = std::fs::read_to_string("shaders/triangle.vert").unwrap();
    let fragment = std::fs::read_to_string("shaders/screen.frag").unwrap();
    rg::ShaderProgramBuilder::vertex_and_fragment(
        "TriangleShader",
        &rg::ShaderCode {
            entry_point: "main".into(),
            data: rg::ShaderData::Glsl(vertex),
        },
        &rg::ShaderCode {
            entry_point: "main".into(),
            data: rg::ShaderData::Glsl(fragment),
        },
    )
    .build(device)
    .unwrap()
}
fn blue_shader(device: &Arc<rg::Device>) -> Arc<rg::Shader> {
    let vertex = std::fs::read_to_string("examples/shaders/screenSpaceQuad.vert").unwrap();
    let fragment = std::fs::read_to_string("shaders/blue.frag").unwrap();
    rg::ShaderProgramBuilder::vertex_and_fragment(
        "BlueShader",
        &rg::ShaderCode {
            entry_point: "main".into(),
            data: rg::ShaderData::Glsl(vertex),
        },
        &rg::ShaderCode {
            entry_point: "main".into(),
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

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)?;

    let mut gfx = rg::Gfx::new(
        &window,
        rg::GfxConfig {
            debug: true,
            features: rg::Features::default(),
        },
    )?;

    let shader = triangle_shader(gfx.device());
    let blue = blue_shader(gfx.device());

    let mut gb: rg::GraphBuilder<(), (), ()> = rg::GraphBuilder::new(&mut gfx, WIDTH, HEIGHT);

    let mut frame_count = 0;
    let mut state = false;

    let blue_target =
        gb.create_image("blue", rg::ImageConfig::color2d(), rg::ImageSize::default())?;

    gb.add_renderpass(
        rg::Renderpass::new("Blue", rg::ImageSize::default(), move |cmd, _, _, _| {
            cmd.set_shader(&blue);
            cmd.draw(0..6, 0..1);
        })
        .draw_image(&blue_target, rg::AttachmentConfig::color_default(1)),
    );

    gb.add_renderpass(
        rg::Renderpass::new("Triangle", rg::ImageSize::default(), move |cmd, _, _, _| {
            cmd.set_shader(&shader);

            if frame_count % 120 == 0 {
                state = !state;
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
        .sample_image(
            &blue_target,
            rg::AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
            3,
        )
        .present(),
    );

    let mut graph = gb.build()?;

    event_loop.run(move |event, _, control_flow| {
        hikari_dev::profile_scope!("mainloop");

        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => {
                graph.execute(&mut gfx, &(), &(), &()).unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id: _,
            } => {
                println!("Closing");
                *control_flow = ControlFlow::Exit;
            }
            Event::LoopDestroyed => {
                graph.prepare_exit();
            }
            _ => (),
        }

        hikari_dev::finish_frame!();
    })
}
