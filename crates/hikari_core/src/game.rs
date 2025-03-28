use std::{sync::Arc, path::Path};

use hikari_asset::{Asset, AssetManager, Loader, Saver, AssetManagerBuilder};
use hikari_systems::*;

use rayon::ThreadPoolBuilder;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::Plugin;

pub type InitResult = anyhow::Result<()>;
pub struct Game {
    event_loop: EventLoop<()>,
    state: StateBuilder,
    init_schedule: ScheduleBuilder<InitResult>,
    run_schedule: ScheduleBuilder<()>,
    exit_schedule: ScheduleBuilder<()>,
    asset_manager_builder: AssetManagerBuilder,
    event_hooks:
        Vec<Box<dyn FnMut(&GlobalState, &mut Window, &Event<()>, &mut ControlFlow) + 'static>>,
}
impl Game {
    pub fn new(window_builder: WindowBuilder) -> Result<Self, winit::error::OsError> {
        hikari_dev::profiling_init();
        let event_loop = EventLoop::new();
        let window = window_builder.build(&event_loop)?;
        let mut state = StateBuilder::new();
        state.add_state(window);

        Ok(Self {
            state,
            event_loop,
            init_schedule: ScheduleBuilder::new(),
            run_schedule: ScheduleBuilder::new(),
            exit_schedule: ScheduleBuilder::new(),
            asset_manager_builder: AssetManager::builder(),
            event_hooks: Vec::new(),
        })
    }
    pub fn add_state(&mut self, state: impl State) -> &mut Self {
        self.state.add_state(state);

        self
    }
    pub fn get<S: State>(&self) -> SysRef<S> {
        self.state.get()
    }
    pub fn get_mut<S: State>(&self) -> SysRefMut<S> {
        self.state.get_mut()
    }
    pub fn create_init_stage(&mut self, name: &str) -> &mut Self {
        self.init_schedule.create_stage(name);

        self
    }
    pub fn create_stage(&mut self, name: &str) -> &mut Self {
        self.run_schedule.create_stage(name);

        self
    }
    pub fn create_exit_stage(&mut self, name: &str) -> &mut Self {
        self.exit_schedule.create_stage(name);

        self
    }
    pub fn add_task(&mut self, stage: &str, task: Task<()>) -> &mut Self {
        self.run_schedule.add_task(stage, task);
        self
    }
    pub fn add_init_task(&mut self, stage: &str, task: Task<InitResult>) -> &mut Self {
        self.init_schedule.add_task(stage, task);
        self
    }
    pub fn add_exit_task(&mut self, stage: &str, task: Task<()>) -> &mut Self {
        self.exit_schedule.add_task(stage, task);
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
    pub fn create_asset<T: Asset>(&mut self) -> &mut Self {
        self.asset_manager_builder.register_asset_type::<T>();

        let mut task_name = String::from(std::any::type_name::<T>());
        task_name.push_str("_asset_update");
        self.add_task(
            crate::LAST,
            Task::new(&task_name, |asset_manager: &AssetManager| {
                asset_manager.update::<T>();
            }),
        );
        self
    }
    pub fn register_asset_loader<T: Asset, L: Loader>(&mut self, loader: L) -> &mut Self {
        self.asset_manager_builder.register_loader::<T, L>(loader);

        self
    }
    pub fn register_asset_saver<T: Asset, S: Saver>(&mut self, saver: S) -> &mut Self {
        self.asset_manager_builder.register_saver::<T, S>(saver);

        self
    }
    pub fn set_asset_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.asset_manager_builder.set_asset_dir(path);

        self
    }
    pub fn window(&mut self) -> SysRefMut<Window> {
        self.state.get_mut::<Window>()
    }
    pub fn run(mut self) -> ! {
        let mut init = self
            .init_schedule
            .build()
            .expect("Failed to create init schedule");
        let mut update = self
            .run_schedule
            .build()
            .expect("Failed to create update schedule");
        let mut exit = self.exit_schedule
        .build()
        .expect("Failed to create exit schedule");

        let asset_manager = {
            let threadpool = Arc::new(ThreadPoolBuilder::new().num_threads(2).build().unwrap());
            self.asset_manager_builder.thread_pool(&threadpool);
            self.asset_manager_builder
                .build()
                .expect("Failed to create asset manager")
        };

        self.state.add_state(asset_manager);

        let mut state = self.state.build();
        let mut hooks = self.event_hooks;

        let event_loop = self.event_loop;

        for result in init.execute_iter(&mut state) {
            result.expect("Failed to Initialize Game");
        }

        event_loop.run(move |event, _, control_flow| {
            {
                let mut window = state.get_mut::<Window>().unwrap();
                for hook in &mut hooks {
                    (hook)(&state, &mut window, &event, control_flow);
                }
            }
            match &event {
                Event::RedrawRequested(_) => {
                    hikari_dev::profile_scope!("Gameloop");
                    update.execute(&mut state);
                    hikari_dev::finish_frame!();
                }
                Event::MainEventsCleared => {
                    let window = state.get_mut::<Window>().unwrap();
                    window.request_redraw();
                }
                Event::LoopDestroyed => {
                    exit.execute(&mut state);
                }
                _ => {}
            }
        })
    }
}
