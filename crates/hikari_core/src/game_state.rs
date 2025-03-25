#[derive(Default)]
pub struct GameState {
    exit_requested: bool,
    frames_since_exit_request: usize
}

impl GameState {
    pub fn request_exit(&mut self) {
        self.exit_requested = true;
    }
    pub fn is_exit_requested(&self) -> bool {
        self.exit_requested
    }
    pub fn can_safely_exit(&self) -> bool {
        self.exit_requested && self.frames_since_exit_request > 1
    }
    pub fn new_frame(&mut self) {
        if self.exit_requested {
            self.frames_since_exit_request += 1;
        }
    }
}