#![allow(dead_code)]

use crate::{
    input::InputState,
    scenerenderer::SceneRendererState,
    window::{EventIter, EventPump, Window},
};
pub struct Context {
    pub(crate) renderer: SceneRendererState,
    pub(crate) input: InputState,
    //pub ui: UI,
    pub(crate) window: Window,
    pub(crate) events: EventPump,

    pub(crate) gfx: graphy::Gfx,

    pub(crate) auto_resize: bool,
}
impl Context {
    pub(crate) fn is_running(&self) -> bool {
        self.window.is_open()
    }
    pub(crate) fn get_window_events(&mut self) -> EventIter {
        self.events.poll_events()
    }
    pub fn set_auto_resize(&mut self, yes: bool) {
        if yes {
            let (width, height) = self.window.get_size();
            self.auto_resize = true;

            self.on_viewport_resize(width, height);
        } else {
            self.auto_resize = false;
        }
    }
    pub fn on_viewport_resize(&mut self, width: u32, height: u32) {
        crate::scenerenderer::on_viewport_resize(self, width, height);
    }
    pub fn exit(&mut self) {
        self.window.request_close();
    }
}

impl Context {
    pub fn window(&mut self) -> &Window {
        &self.window
    }
    pub fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
    pub fn gfx(&mut self) -> &graphy::Gfx {
        &self.gfx
    }
    pub fn gfx_mut(&mut self) -> &mut graphy::Gfx {
        &mut self.gfx
    }
}
