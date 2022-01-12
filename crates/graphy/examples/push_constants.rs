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

const QUAD_VERTS: [f32; 4 * 2] = [0.5, 0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5];

const QUAD_INDS: [u32; 6] = [0, 1, 2, 0, 2, 3];

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

    let mut vertices = rg::create_vertex_buffer::<f32>(gfx.device(), QUAD_VERTS.len())?;
    vertices.upload(&QUAD_VERTS, 0)?;

    let mut indices = rg::create_index_buffer(gfx.device(), QUAD_INDS.len())?;
    indices.upload(&QUAD_INDS, 0)?;

    let pbr = rg::ShaderProgramBuilder::vertex_and_fragment(
        "Quad",
        &rg::ShaderCode {
            entry_point: "main".into(),
            data: rg::ShaderData::Glsl(std::fs::read_to_string(
                "examples/shaders/push_constant.vert",
            )?),
        },
        &rg::ShaderCode {
            entry_point: "main".into(),
            data: rg::ShaderData::Glsl(std::fs::read_to_string(
                "examples/shaders/push_constant.frag",
            )?),
        },
    )
    .build(gfx.device())?;

    let mut gb: rg::GraphBuilder<f32, (), ()> = rg::GraphBuilder::new(&mut gfx, WIDTH, HEIGHT);

    let layout = rg::VertexInputLayout::new()
        .buffer(&[rg::ShaderDataType::Vec2f], rg::StepMode::Vertex) // Binding 0
        //.buffer(...)                                                    // Binding 1
        //...
        //.buffer(...)                                                    // Binding n (upto 4 bindings supported)
        .build();

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct PushConstants {
        color: [f32; 4],
        position: glam::Vec2,
    }

    let mut t = 0;
    gb.add_renderpass(
        rg::Renderpass::new("Push", rg::ImageSize::default(), move |cmd, _, _, _| {
            cmd.set_shader(&pbr);
            cmd.set_vertex_input_layout(layout);
            cmd.set_vertex_buffer(&vertices, 0);
            cmd.set_index_buffer(&indices);

            cmd.push_constants(
                &PushConstants {
                    position: glam::vec2(1.0 * f32::sin(t as f32 * 0.01), 0.0),
                    color: glam::vec4(1.0, 0.0, 0.0, 1.0).into(),
                },
                0,
            );
            cmd.draw_indexed(0..QUAD_INDS.len(), 0, 0..1);

            cmd.push_constants(
                &PushConstants {
                    position: glam::vec2(0.0, 1.0 * f32::sin(t as f32 * 0.01)),
                    color: glam::vec4(0.0, 0.0, 0.59, 1.0).into(),
                },
                0,
            );
            cmd.draw_indexed(0..QUAD_INDS.len(), 0, 0..1);

            t += 1;
        })
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

                graph.execute(&mut gfx, &dt, &(), &()).unwrap();

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
