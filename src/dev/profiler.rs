#[macro_export(local_inner_macros)]
macro_rules! profile_scope {
    ($x: expr) => {
        #[cfg(feature = "profile")]
        let timer = $crate::dev::timer::Timer::start($x);
    };
}
#[macro_export(local_inner_macros)]
macro_rules! profile_function {
    () => {
        #[cfg(feature = "profile")]
        let timer = {
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);
            let name = &name[0..name.len() - 3];
            $crate::dev::timer::Timer::start(name)
        };
    };
}
use std::io::prelude::*;
use std::{fs::File, io::BufWriter, path::Path};

use parking_lot::Mutex;

use serde::Serialize;
pub struct ProfileResult {
    pub name: &'static str,
    pub thread_id: u32,
    pub start_time: u128,
    pub duration: u128,
}
#[derive(Serialize)]
struct ChromeTraceEvent {
    cat: &'static str,
    dur: u128,
    name: &'static str,
    ph: &'static str,
    pid: u32,
    tid: u32,
    ts: u128,
}
struct Profiler {
    file_path: &'static str,
    stream: BufWriter<File>,
    is_session_active: bool,

    first: bool,
}
impl Profiler {
    fn init(file_path: &'static str) -> Self {
        let file = File::create(Path::new(file_path))
            .expect(format!("Failed to write to {}", file_path).as_str());
        let stream = BufWriter::new(file);

        Self {
            file_path,
            stream,
            is_session_active: false,
            first: true,
        }
    }
    fn begin_session(&mut self) {
        self.is_session_active = true;
        self.write_header();
    }
    fn end_session(&mut self) {
        self.write_footer();
        self.is_session_active = false;
        self.first = true;
    }

    fn write_header(&mut self) {
        self.stream
            .write(b"{ \"otherData\": {}, \"traceEvents\":[")
            .expect(format!("Failed to write to {}", self.file_path).as_str());
    }
    fn write_footer(&mut self) {
        self.stream
            .write(b"]}")
            .expect(format!("Failed to write to {}", self.file_path).as_str());
        self.stream.flush().unwrap();
    }
    fn submit_data(&mut self, data: ProfileResult) {
        if !self.first {
            self.stream.write(b",").unwrap();
        } else {
            self.first = false;
        }

        let trace_event = ChromeTraceEvent {
            cat: "function",
            dur: data.duration,
            name: data.name,
            ph: "X",
            pid: 0,
            tid: data.thread_id,
            ts: data.start_time,
        };
        let json_data = serde_json::to_vec(&trace_event).expect("Failed to serialize");

        self.stream
            .write(json_data.as_slice())
            .expect(format!("Failed to write to {}", self.file_path).as_str());
    }
}
lazy_static! {
    static ref PROFILER: Mutex<Profiler> = Mutex::new(Profiler::init("profile.json"));
}

pub fn submit(result: ProfileResult) {
    let mut profiler_locked = PROFILER.lock();
    if profiler_locked.is_session_active {
        profiler_locked.submit_data(result);
    } else {
        println!("No active sessions.. Doing nothing");
    }
}
pub fn begin_session() {
    let mut profiler_locked = PROFILER.lock();
    profiler_locked.begin_session();
}
pub fn end_session() {
    let mut profiler_locked = PROFILER.lock();

    if profiler_locked.is_session_active {
        profiler_locked.end_session();
    } else {
        println!("No active sessions.. Doing nothing");
    }
}
