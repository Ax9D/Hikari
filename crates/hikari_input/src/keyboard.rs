use hikari_core::winit::event::WindowEvent;

pub type KeyCode = hikari_core::winit::event::VirtualKeyCode;
pub type KeyState = hikari_core::winit::event::ElementState;

pub struct KeyboardState {
    last: Vec<KeyState>,
    current: Vec<KeyState>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            last: vec![KeyState::Released; 163],
            current: vec![KeyState::Released; 163],
        }
    }
    pub(crate) fn update(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(code) = input.virtual_keycode {
                    self.current[code as usize] = input.state;
                }
            }
            _ => {}
        }
    }
    pub(crate) fn new_frame(&mut self) {
        self.last = self.current.clone();
    }

    #[inline]
    pub fn get_key_state(&self, key: KeyCode) -> KeyState {
        self.current[key as usize]
    }
    #[inline]
    pub fn get_last_key_state(&self, key: KeyCode) -> KeyState {
        self.last[key as usize]
    }
    #[inline]
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.get_key_state(key) == KeyState::Pressed
    }
    #[inline]
    pub fn is_key_up(&self, key: KeyCode) -> bool {
        self.get_key_state(key) == KeyState::Released
    }
    #[inline]
    pub fn was_just_pressed(&self, key: KeyCode) -> bool {
        self.get_key_state(key) == KeyState::Pressed
            && self.get_last_key_state(key) == KeyState::Released
    }
    #[inline]
    pub fn was_just_released(&self, key: KeyCode) -> bool {
        self.get_key_state(key) == KeyState::Released
            && self.get_last_key_state(key) == KeyState::Pressed
    }
}

#[test]
pub fn was_just_pressed() {
    use hikari_core::winit::event::*;
    #[allow(deprecated)]
    fn dummy_event(code: KeyCode, state: KeyState) -> WindowEvent<'static> {
        WindowEvent::KeyboardInput {
            device_id: unsafe { DeviceId::dummy() },
            input: KeyboardInput {
                scancode: 0,
                state,
                virtual_keycode: Some(code),
                modifiers: ModifiersState::empty(),
            },
            is_synthetic: true,
        }
    }
    let mut keyboard = KeyboardState::new();
    keyboard.update(&dummy_event(KeyCode::A, KeyState::Released));
    keyboard.new_frame();
    keyboard.update(&dummy_event(KeyCode::A, KeyState::Pressed));

    assert!(keyboard.was_just_pressed(KeyCode::A));
    assert!(!keyboard.was_just_released(KeyCode::A));
    assert!(!keyboard.was_just_pressed(KeyCode::B));
}
