// use graphy::{Graph, Gfx};
// use winit::{event_loop::{self, EventLoop, ControlFlow}, window::{WindowBuilder, Window}, dpi::LogicalSize, event::{Event, WindowEvent}};

// pub struct GameLoop {
//     pub window: Window,
//     pub event_loop: EventLoop<()>
// }

// impl GameLoop {
//     pub fn new(window_builder: WindowBuilder) -> Result<Self, Box<dyn std::error::Error>> {
//         let event_loop = EventLoop::new();
//         let window = window_builder.build(&event_loop)?;

//         Ok(Self {
//             window, 
//             event_loop
//         })
//     }
//     pub fn run<S, P, R>(mut self, gfx: &Gfx, graph: &mut Graph<S,P,R>, mut run: impl FnMut(&mut Window, Event<()>, &mut ControlFlow) + 'static) -> ! {
//         self.event_loop.run(move |event, _, control_flow| {
//             hikari_dev::profile_scope!("mainloop");
            
//             *control_flow = ControlFlow::Poll;
            
//             match event {
//                 Event::MainEventsCleared => {
//                     graph.execute(&mut gfx, &(), &(), &()).unwrap();
//                     (run)(&mut self.window, event, control_flow);
//                 }
//                 Event::WindowEvent {
//                     event: WindowEvent::CloseRequested,
//                     window_id: _,
//                 } => {
//                     println!("Closing");
//                     *control_flow = ControlFlow::Exit;
//                 }
//                 Event::LoopDestroyed => {
//                     graph.prepare_exit();
//                 }
//                 _ => (),
//             }
    
//             hikari_dev::finish_frame!();
//         })
//     }
// }