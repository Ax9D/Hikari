use std::{thread::ThreadId, time::SystemTime};

use super::profiler::{self, ProfileResult};
pub struct Timer {
    name: &'static str,
    start_time: SystemTime,
}
impl Timer {
    pub fn start(name: &'static str) -> Self {
        let start_time = SystemTime::now();
        Self { name, start_time }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let start_time = self
            .start_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();

        let duration = self.start_time.elapsed().unwrap().as_micros();

        let thread_id = std::thread::current().id();

        let thread_id = unsafe { *((&thread_id as *const ThreadId) as *const u64) };

        let profile_result = ProfileResult {
            name: self.name,
            thread_id: thread_id as u32,
            start_time,
            duration,
        };
        profiler::submit(profile_result);
    }
}
