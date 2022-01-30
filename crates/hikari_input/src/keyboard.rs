use winit::event::WindowEvent;

pub type KeyCode = winit::event::VirtualKeyCode;
pub type KeyState = winit::event::ElementState;

pub struct KeyboardState {
    keys: Vec<KeyState>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            keys: vec![KeyState::Released; 163],
        }
    }
    pub(crate) fn update(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(code) = input.virtual_keycode {
                    self.keys[code as usize] = input.state;
                }
            }
            _ => {}
        }
    }

    #[inline]
    pub fn get_key_state(&self, key: KeyCode) -> KeyState {
        self.keys[key as usize]
    }
    #[inline]
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.get_key_state(key) == KeyState::Pressed
    }
    #[inline]
    pub fn is_released(&self, key: KeyCode) -> bool {
        self.get_key_state(key) == KeyState::Released
    }
}
