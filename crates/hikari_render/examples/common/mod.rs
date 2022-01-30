use hikari_render::{Gfx, Graph, GfxConfig};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{self, ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct GameLoop {
    pub window: Window,
    pub event_loop: EventLoop<()>,
}

impl GameLoop {
    pub fn new(
        window_builder: WindowBuilder,
        config: GfxConfig,
    ) -> Result<(Gfx, Self), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new();
        let window = window_builder.build(&event_loop)?;

        Ok((Gfx::new(&window, config)?, Self { window, event_loop }))
    }
    pub fn run(
        mut self,
        mut gfx: Gfx,
        mut run: impl FnMut(&mut Gfx, &mut Window, Event<()>, &mut ControlFlow) + 'static,
    ) -> ! {
        self.event_loop.run(move |event, _, control_flow| {
            hikari_dev::profile_scope!("Gameloop");
            *control_flow = ControlFlow::Poll;

            match &event {
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                Event::WindowEvent {
                    event,
                    window_id: _,
                } => match event {
                    WindowEvent::Resized(size) => {
                        gfx.resize(size.width, size.height)
                            .expect("Failed to resize graphics context");
                    }
                    WindowEvent::CloseRequested => {
                        println!("Closing");
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                },
                Event::LoopDestroyed => {}
                _ => (),
            }

            (run)(&mut gfx, &mut self.window, event, control_flow);
            hikari_dev::finish_frame!();
        })
    }
}
