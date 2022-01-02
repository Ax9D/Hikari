mod profile;
pub use profile::*;

pub use profiling;

#[macro_export]
macro_rules! profile_scope {
    ($name: expr) => {
        $crate::profiling::scope!($name);
    };

    ($name: expr, $data: expr) => {
        $crate::profiling::scope!($name, $data);
    }
}
#[macro_export]
macro_rules! profile_function {
    () => {
        $crate::profile_scope!($crate::function!()) 
    }
}

#[macro_export]
macro_rules! function {
    () => {
        {
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);
            let name = &name[..name.len() - 3];
    
            $crate::clean_function_name(name)
         }
    };
}