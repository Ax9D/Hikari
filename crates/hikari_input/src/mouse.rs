use winit::{dpi::PhysicalPosition, event::WindowEvent};
pub type MouseButtonState = winit::event::ElementState;
pub type MouseButton = winit::event::MouseButton;

pub struct MouseState {
    position: PhysicalPosition<f64>,
    cur_delta: glam::Vec2,
    scroll_delta: glam::Vec2,

    buttons: fxhash::FxHashMap<MouseButton, MouseButtonState>,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            position: PhysicalPosition { x: 0.0, y: 0.0 },
            cur_delta: glam::Vec2::ZERO,
            scroll_delta: glam::Vec2::ZERO,
            buttons: Default::default(),
        }
    }
    pub(crate) fn update(&mut self, event: &WindowEvent) {
        let prev_position = self.position;
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.position = *position;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                *self
                    .buttons
                    .entry(*button)
                    .or_insert(MouseButtonState::Released) = *state;
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    self.scroll_delta.x = *x;
                    self.scroll_delta.y = *y;
                }
                winit::event::MouseScrollDelta::PixelDelta(_) => todo!(),
            },
            _ => {}
        }

        self.cur_delta.x = (self.position.x - prev_position.x) as f32;
        self.cur_delta.y = (self.position.y - prev_position.y) as f32;
    }

    #[inline]
    pub fn get_position(&self) -> glam::Vec2 {
        glam::vec2(self.position.x as f32, self.position.y as f32)
    }
    #[inline]
    pub fn get_cursor_delta(&self) -> glam::Vec2 {
        self.cur_delta
    }
    #[inline]
    pub fn get_scroll_delta(&self) -> glam::Vec2 {
        self.scroll_delta
    }
    pub fn get_button_state(&self, button: MouseButton) -> MouseButtonState {
        *self
            .buttons
            .get(&button)
            .unwrap_or(&MouseButtonState::Released)
    }
    #[inline]
    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.get_button_state(button) == MouseButtonState::Pressed
    }
    #[inline]
    pub fn is_released(&self, button: MouseButton) -> bool {
        self.get_button_state(button) == MouseButtonState::Released
    }
}
