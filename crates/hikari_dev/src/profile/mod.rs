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

#[cfg(not(any(
        feature = "profiling_tracy",
        feature = "profiling_optick"
    )
))]
mod noop;
#[cfg(not(any(
    feature = "profiling_tracy",
    feature = "profiling_optick"
)
))]
pub use noop::*;

#[cfg(feature = "profiling_tracy")]
mod tracy_impl;
#[cfg(feature = "profiling_tracy")]
pub use tracy_impl::*;

#[cfg(feature = "profiling_optick")]
mod optick_impl;
#[cfg(feature = "profiling_optick")]
pub use optick_impl::*;