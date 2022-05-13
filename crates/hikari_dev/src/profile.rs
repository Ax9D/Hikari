#[inline]
pub fn clean_function_name(name: &str) -> &str {
    if let Some(colon) = name.rfind("::") {
        if let Some(colon) = name[..colon].rfind("::") {
            // "foo::bar::baz::function_name" -> "baz::function_name"
            &name[colon + 2..]
        } else {
            // "foo::function_name" -> "foo::function_name"
            name
        }
    } else {
        name
    }
}

#[cfg(not(feature = "profiling_disabled"))]
pub use tracy_client;
#[cfg(not(feature = "profiling_disabled"))]
mod tracy {
    #[macro_export]
    macro_rules! profile_scope {
        ($name: expr) => {
            let _tracy_span = $crate::tracy_client::Client::running().expect("tracy_client::Client is not running!").span_alloc($name, "", file!(), line!(), 0);
        };

        ($name: expr, $data: expr) => {
            let _tracy_span = $crate::tracy_client::Client::running().expect("tracy_client::Client is not running!").span_alloc($name, "", file!(), line!(), 0);
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
    #[macro_export]
    macro_rules! function {
        () => {{
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);
            let name = &name[..name.len() - 3];

            $crate::clean_function_name(name)
        }};
    }
    pub fn profiling_init() {
        tracy_client::Client::start();
    }
}
#[cfg(feature = "profiling_disabled")]
mod noop {
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
    #[macro_export]
    macro_rules! function {
        () => {};
    }
    pub fn profiling_init() {}
}

#[cfg(not(feature = "profiling_disabled"))]
pub use tracy::*;
#[cfg(feature = "profiling_disabled")]
pub use noop::*;