pub use ::optick;

#[macro_export]
macro_rules! profile_scope {
    ($name: expr) => {
        $crate::optick::event!($name);
    };

    ($name: expr, $data: expr) => {
        $crate::optick::event!($name);
        $crate::optick::tag!("tag", $data);
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
        $crate::optick::next_frame();
    };
}

pub fn profiling_init() {}
