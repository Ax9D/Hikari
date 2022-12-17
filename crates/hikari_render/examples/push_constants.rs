use simple_logger::SimpleLogger;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use hikari_render as rg;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const QUAD_VERTS: [f32; 4 * 2] = [0.5, 0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5];

const QUAD_INDS: [u32; 6] = [2, 1, 0, 3, 2, 0];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .without_timestamps()
        //.with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    hikari_dev::profiling_init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)?;

    let mut gfx = rg::Gfx::new(
        &window,
        rg::GfxConfig {
            debug: true,
            features: rg::Features::default(),
            ..Default::default()
        },
    )?;

    let mut vertices = rg::create_vertex_buffer::<f32>(gfx.device(), QUAD_VERTS.len())?;
    vertices.upload(&QUAD_VERTS, 0)?;

    let mut indices = rg::create_index_buffer(gfx.device(), QUAD_INDS.len())?;
    indices.upload(&QUAD_INDS, 0)?;

    let pbr = rg::ShaderProgramBuilder::vertex_and_fragment(
        "Quad",
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(std::fs::read_to_string(
                "examples/shaders/push_constant.vert",
            )?),
        },
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(std::fs::read_to_string(
                "examples/shaders/push_constant.frag",
            )?),
        },
    )
    .build(gfx.device())?;

    let mut gb: rg::GraphBuilder<(f32, f32)> = rg::GraphBuilder::new(&mut gfx, WIDTH, HEIGHT);

    let layout = rg::VertexInputLayout::builder()
        .buffer(&[rg::ShaderDataType::Vec2f], rg::StepMode::Vertex) // Binding 0
        //.buffer(...)                                                    // Binding 1
        //...
        //.buffer(...)                                                    // Binding n (upto 4 bindings supported)
        .build();

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    struct PushConstants {
        color: [f32; 4],
        position: hikari_math::Vec2,
    }

    let mut t = 0;
    gb.add_renderpass(
        rg::Renderpass::<(f32, f32)>::new("Push", rg::ImageSize::default_xy())
            .cmd(
                move |cmd: &mut rg::RenderpassCommands, _, record_info, (_, _)| {
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

                    cmd.set_shader(&pbr);
                    cmd.set_vertex_input_layout(layout);
                    cmd.set_vertex_buffer(&vertices, 0);
                    cmd.set_index_buffer(&indices);

                    cmd.push_constants(
                        &PushConstants {
                            position: hikari_math::vec2(1.0 * f32::sin(t as f32 * 0.01), 0.0),
                            color: hikari_math::vec4(1.0, 0.0, 0.0, 1.0).into(),
                        },
                        0,
                    );
                    cmd.draw_indexed(0..QUAD_INDS.len(), 0, 0..1);

                    cmd.push_constants(
                        &PushConstants {
                            position: hikari_math::vec2(0.0, 1.0 * f32::sin(t as f32 * 0.01)),
                            color: hikari_math::vec4(0.0, 0.0, 0.59, 1.0).into(),
                        },
                        0,
                    );
                    cmd.draw_indexed(0..QUAD_INDS.len(), 0, 0..1);

                    t += 1;
                },
            )
            .present(),
    );
    let mut dt = 0.0;

    let mut graph = gb.build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                hikari_dev::profile_scope!("mainloop");
                let now = std::time::Instant::now();

                graph.execute((&dt, &dt)).unwrap();

                dt = now.elapsed().as_secs_f32();
                hikari_dev::finish_frame!();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
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
    })
}
