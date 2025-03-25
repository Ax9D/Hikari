pub use tracy_client;

#[macro_export]
macro_rules! profile_scope {
    ($name: literal) => {
        let _tracy_span = $crate::tracy_client::span!($name, 0);
    };
    ($name: literal, $data: expr) => {
        let _tracy_span = $crate::tracy_client::span!($name, 0);
        _tracy_span.emit_text($data);
    };
    ($name: expr) => {
        let _tracy_span = $crate::tracy_client::Client::running()
            .expect("tracy_client::Client is not running!")
            .span_alloc($name, "", file!(), line!(), 0);
    };
    ($name: expr, $data: expr) => {
        let _tracy_span = $crate::tracy_client::Client::running()
            .expect("tracy_client::Client is not running!")
            .span_alloc($name, "", file!(), line!(), 0);
        _tracy_span.emit_text($data);
    };
}
#[macro_export]
macro_rules! profile_function {
    () => {
        $crate::profile_scope!($crate::function!())
    };
}
#[macro_export]
macro_rules! finish_frame {
    () => {
        $crate::tracy_client::Client::running()
            .expect("finish_frame! without a running tracy_client::Client")
            .frame_mark();
    };
}
pub fn profiling_init() {
    tracy_client::Client::start();
}