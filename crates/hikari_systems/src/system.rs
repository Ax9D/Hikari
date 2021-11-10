
pub struct Type {
    name: &'static str,
    id: TypeId,
}

pub struct Function {
    param_types: Vec<Type>,
    exec: Box<dyn FnMut(&UnsafeGlobalState) + 'static>,
}
impl Function {
    #[inline]
    pub unsafe fn run(&mut self, g_state: &UnsafeGlobalState) {
        
        (self.exec)(g_state);
    }
}
pub trait IntoFunction<Params>: 'static {
    fn into_function(self) -> Function;
}

use std::any::TypeId;
use std::any::type_name;

use crate::global::UnsafeGlobalState;
use crate::query::Fetch;

use crate::query::Query;

macro_rules! impl_into_system {
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
impl_into_system!();
impl_into_system!(A);
impl_into_system!(A, B);
impl_into_system!(A, B, C);
impl_into_system!(A, B, C, D);
impl_into_system!(A, B, C, D, E);
impl_into_system!(A, B, C, D, E, F);
impl_into_system!(A, B, C, D, E, F, G);
impl_into_system!(A, B, C, D, E, F, G, H);