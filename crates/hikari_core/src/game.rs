use hikari_systems::*;

use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::Plugin;

pub struct Game {
    state: StateBuilder,
    schedule: ScheduleBuilder,
    event_hooks: Vec<Box<dyn HookFn>>,
}

pub trait HookFn: FnMut(&GlobalState, &Event<()>, &mut ControlFlow) + 'static {}

impl Game {
    pub fn with_defaults() -> Self {
        Self {
            state: StateBuilder::new(),
            schedule: ScheduleBuilder::new(),
            event_hooks: Vec::new(),
        }
    }
    pub fn add_state(&mut self, state: impl State) -> &mut Self {
        self.state.add_state(state);

        self
    }
    pub fn get<S: State>(&self) -> Ref<S> {
        self.state.get()
    }
    pub fn get_mut<S: State>(&self) -> RefMut<S> {
        self.state.get_mut()
    }
    pub fn add_stage(&mut self, stage: Stage) -> &mut Self {
        self.schedule.add_stage(stage);

        self
    }
    pub fn add_function<Params>(&mut self, function: impl IntoFunction<Params>) -> &mut Self {
        self.schedule.add_to_stage("Update", function);

        self
    }
    pub fn add_function_to_stage<Params>(
        &mut self,
        stage: &str,
        function: impl IntoFunction<Params>,
    ) -> &mut Self {
        self.schedule.add_to_stage(stage, function);

        self
    }
    pub fn add_platform_event_hook(&mut self, hook: impl HookFn) -> &mut Self {
        self.event_hooks.push(Box::new(hook));
        self
    }
    pub fn add_plugin(&mut self, mut plugin: impl Plugin + 'static) -> &mut Self {
        plugin.build(self);
        self
    }
}
pub struct GameLoop {
    window: Window,
    event_loop: EventLoop<()>,
}

impl GameLoop {
    pub fn new(window_builder: WindowBuilder) -> Result<Self, Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new();
        let window = window_builder.build(&event_loop)?;

        Ok(Self { window, event_loop })
    }
    pub fn run(self, game: Game) -> ! {
        let mut update = game
            .schedule
            .build()
            .expect("Failed to create update schedule");
        let mut state = game.state.build();
        let mut hooks = game.event_hooks;

        let window_builder = unsafe {state.get::<WindowBuilder>().unwrap()};
        let window_builder = window_builder.clone();
        let event_loop = EventLoop::new();
        let window = window_builder.build(&event_loop)?;

        event_loop.run(move |event, _, control_flow| {
            hikari_dev::profile_scope!("Gameloop");
            *control_flow = ControlFlow::Poll;

            {
                for hook in &mut hooks {
                    (hook)(&state, &event, control_flow);
                }
            }

            match &event {
                Event::RedrawRequested(_) => {
                    update.execute(&mut state);
                }
                Event::MainEventsCleared => {
                    let window = unsafe { state.get_mut::<Window>().unwrap() };
                    window.request_redraw();
                }
                Event::LoopDestroyed => {}
                _ => {}
            }
            hikari_dev::finish_frame!();
        })
    }
}
