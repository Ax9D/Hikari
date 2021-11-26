#[allow(dead_code)]
pub struct Type {
    name: &'static str,
    id: TypeId,
}
#[allow(dead_code)]
pub struct Function {
    param_types: Vec<Type>,
    exec: Box<dyn FnMut(Pin<&UnsafeGlobalState>) + 'static>,
}
impl Function {
    #[inline]
    pub unsafe fn run(&mut self, g_state: Pin<&UnsafeGlobalState>) {  
        (self.exec)(g_state);
    }
}
pub trait IntoFunction<Params>: 'static {
    fn into_function(self) -> Function;
}

use std::any::TypeId;
use std::any::type_name;
use std::pin::Pin;

use crate::global::UnsafeGlobalState;
use crate::query::Fetch;

use crate::query::Query;

macro_rules! impl_into_function {
    ($($name: ident),*) => {
        #[allow(non_snake_case)]
        impl<'a, Func, Return, $($name: Query + 'static),*> IntoFunction<($($name,)*)> for Func
        where
            Func:
                FnMut($($name),*) -> Return +
                FnMut($(<<$name as Query>::Fetch as Fetch>::Item),* ) -> Return +
                Send + Sync + 'static {
            fn into_function(mut self) -> Function {
                Function {
                    exec: Box::new(move |g_state| {
                        let ($($name,)*) = unsafe { g_state.query::<($($name,)*)>() };

                        self($($name,)*);
                    }),
                    param_types: vec![$(
                        Type {
                            name: type_name::<$name>(),
                            id: TypeId::of::<$name>()
                        }
                        ,)*]
                }
            }
        }
    };
}
impl_into_function!();
impl_into_function!(A);
impl_into_function!(A, B);
impl_into_function!(A, B, C);
impl_into_function!(A, B, C, D);
impl_into_function!(A, B, C, D, E);
impl_into_function!(A, B, C, D, E, F);
impl_into_function!(A, B, C, D, E, F, G);
impl_into_function!(A, B, C, D, E, F, G, H);