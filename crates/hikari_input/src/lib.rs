pub mod keyboard;
mod mouse;

pub use keyboard::*;
pub use mouse::*;
use winit::event::WindowEvent;

pub struct Input {
    keyboard_state: KeyboardState,
    mouse_state: MouseState
}

impl Input {
    pub fn new() -> Self {
        Self {
            keyboard_state: KeyboardState::new(),
            mouse_state: MouseState::new()
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
}

pub struct InputPlugin