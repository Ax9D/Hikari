use std::path::PathBuf;

pub fn engine_dir() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    exe.parent().unwrap().to_owned()
}