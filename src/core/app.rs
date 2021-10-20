use crate::window::{EventType, Window};

use super::{Context, Plugin};
use std::{
    borrow::{Borrow, BorrowMut},
    error::Error,
    time::Instant,
};
pub struct App {
    context: Context,

    plugins: Vec<Box<dyn Plugin + 'static>>,
}
impl App {
    pub fn new(name: &str, width: u32, height: u32) -> Result<Self, Box<dyn Error>> {
        let (mut window, events) = Window::create(width, height)?;

        window.set_title(name);

        let debug = true; //cfg!(debug_assertions);

        let gfx = graphy::Gfx::new(&mut window.get_raw_mut(), debug)?;

        let renderer = crate::scenerenderer::init(&gfx, &mut window)?;
        let input = crate::input::init();

        let mut context = crate::core::Context {
            renderer,
            input,
            window,
            events,
            auto_resize: true,
            gfx,
        };

        //Do init related stuff
        crate::texture::init(&mut context).unwrap();
        //End of init
        Ok(Self {
            context,
            plugins: Vec::new(),
        })
    }
    pub fn add_plugin<P: Plugin + 'static>(&mut self) {
        self.plugins.push(Box::new(P::on_init(&mut self.context)));
    }
    pub fn run(mut self) {
        let mut prev_time = Instant::now();
        let safe_ctx = (&mut self.context) as *mut _;
        while self.context.is_running() {
            let now = Instant::now();

            let dt = now.duration_since(prev_time).as_secs_f32();
            prev_time = now;

            crate::core::update(&mut self.context);
            //self.gfx.run(&self.world);

            for mut event in self.context.get_window_events() {
                for plugin in self.plugins.iter_mut() {
                    plugin.on_event(unsafe { &mut *safe_ctx }, &mut event);
                }
            }

            for plugin in self.plugins.iter_mut() {
                plugin.on_update(&mut self.context, dt);
            }

            self.context.window.update();
        }
    }
}

pub struct BasePlugin {}
impl Plugin for BasePlugin {
    fn on_init(ctx: &mut Context) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn on_update(&mut self, ctx: &mut Context, dt: f32) {}

    fn on_event(&mut self, ctx: &mut Context, event: &mut crate::window::Event) {
        match event.kind {
            EventType::FramebufferSize(width, height) => {
                if ctx.auto_resize {
                    ctx.on_viewport_resize(width as u32, height as u32);
                }
            }
            _ => {}
        }
    }
}
