#[macro_export]
macro_rules! profile_scope {
    ($name: expr) => {};

    ($name: expr, $data: expr) => {};
}
#[macro_export]
macro_rules! profile_function {
    () => {};
}
#[macro_export]
macro_rules! finish_frame {
    () => {};
}
pub fn profiling_init() {}
