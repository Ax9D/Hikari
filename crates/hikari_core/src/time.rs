use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct Time {
    last: Instant,
    current_dt: Duration,
}
impl Time {
    pub(crate) fn new() -> Self {
        Self {
            last: Instant::now(),
            current_dt: Duration::ZERO,
        }
    }
    pub(crate) fn update(&mut self) {
        let now = Instant::now();

        self.current_dt = now - self.last;

        self.last = now;
    }
    pub fn dt(&self) -> f32 {
        self.current_dt.as_secs_f32()
    }
}
