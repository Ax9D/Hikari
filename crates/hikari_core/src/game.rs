use hikari_systems::*;

use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::Plugin;

pub struct Game {
    window: Window,
    event_loop: EventLoop<()>,
    state: StateBuilder,
    init_schedule: ScheduleBuilder,
    run_schedule: ScheduleBuilder,
    event_hooks:
        Vec<Box<dyn FnMut(&GlobalState, &mut Window, &Event<()>, &mut ControlFlow) + 'static>>,
}
impl Game {
    pub fn new(window_builder: WindowBuilder) -> Result<Self, winit::error::OsError> {
        hikari_dev::profiling_init();
        let event_loop = EventLoop::new();
        let window = window_builder.build(&event_loop)?;
        Ok(Self {
            window,
            event_loop,
            state: StateBuilder::new(),
            init_schedule: ScheduleBuilder::new(),
            run_schedule: ScheduleBuilder::new(),
            event_hooks: Vec::new(),
        })
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
    pub fn create_stage(&mut self, name: &str) -> &mut Self {
        self.run_schedule.create_stage(name);

        self
    }
    pub fn add_task(&mut self, stage: &str, task: Task) -> &mut Self {
        self.run_schedule.add_task(stage, task);
        self
    }
    pub fn add_init_task(&mut self, stage: &str, task: Task) -> &mut Self {
        self.init_schedule.add_task(stage, task);
        self
    }
    pub fn add_platform_event_hook(
        &mut self,
        hook: impl FnMut(&GlobalState, &mut Window, &Event<()>, &mut ControlFlow) + 'static,
    ) -> &mut Self {
        self.event_hooks.push(Box::new(hook));
        self
    }
    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.build(self);
        log::debug!("Successfully added plugin: {}", std::any::type_name::<P>());

        self
    }
    pub fn window(&mut self) -> &mut Window {
        &mut self.window
    }

    pub fn run(self) -> ! {
        let mut init = self
            .init_schedule
            .build()
            .expect("Failed to create init schedule");
        let mut update = self
            .run_schedule
            .build()
            .expect("Failed to create update schedule");
        let mut state = self.state.build();
        let mut hooks = self.event_hooks;

        let event_loop = self.event_loop;
        let mut window = self.window;

        init.execute(&mut state);
        event_loop.run(move |event, _, control_flow| {
            hikari_dev::profile_scope!("Gameloop");

            for hook in &mut hooks {
                (hook)(&state, &mut window, &event, control_flow);
            }

            match &event {
                Event::RedrawRequested(_) => {
                    update.execute(&mut state);
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::LoopDestroyed => {}
                _ => {}
            }
            hikari_dev::finish_frame!();
        })
    }
}
