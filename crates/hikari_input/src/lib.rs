pub mod keyboard;
mod mouse;

use hikari_systems::Task;
pub use keyboard::*;
pub use mouse::*;
use winit::event::WindowEvent;

pub struct Input {
    keyboard_state: KeyboardState,
    mouse_state: MouseState,
}

impl Input {
    pub fn new() -> Self {
        Self {
            keyboard_state: KeyboardState::new(),
            mouse_state: MouseState::new(),
        }
    }
    #[inline]
    pub fn keyboard(&self) -> &KeyboardState {
        &self.keyboard_state
    }
    #[inline]
    pub fn mouse(&self) -> &MouseState {
        &self.mouse_state
    }
    #[inline]
    pub fn update(&mut self, event: &WindowEvent) {
        self.keyboard_state.update(event);
        self.mouse_state.update(event);
    }
    pub fn new_frame(&mut self) {
        self.keyboard_state.new_frame()
    }
}

pub struct InputPlugin;

impl hikari_core::Plugin for InputPlugin {
    fn build(self, game: &mut hikari_core::Game) {
        game.add_state(Input::new());
        game.add_task(
            hikari_core::LAST,
            Task::new("Input New Frame", |input: &mut Input| {
                input.new_frame();
            }),
        );
        game.add_platform_event_hook(|state, _window, event, _control| match event {
            winit::event::Event::WindowEvent { event, .. } => {
                state.get_mut::<Input>().unwrap().update(event);
            }
            _ => {}
        });
    }
}
