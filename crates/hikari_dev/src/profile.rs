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
macro_rules! profile_scope {
    ($name: expr) => {
        $crate::profiling::scope!($name);
    };

    ($name: expr, $data: expr) => {
        $crate::profiling::scope!($name, $data);
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
        $crate::profiling::finish_frame!()
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

pub use finish_frame;
pub use function;
pub use profile_function;
pub use profile_scope;
