use crate::query::Borrows;
use crate::query::Query;

#[allow(dead_code)]
pub struct Function {
    borrows: Borrows,
    exec: Box<dyn FnMut(Pin<&UnsafeGlobalState>) + 'static>,
}
impl Function {
    #[inline]
    pub unsafe fn run(&mut self, g_state: Pin<&UnsafeGlobalState>) {
        (self.exec)(g_state);
    }
}
pub trait IntoFunction<Params> {
    fn into_function(self) -> Function;
}

use std::pin::Pin;

use crate::global::UnsafeGlobalState;
use crate::query::Fetch;

macro_rules! impl_into_function {
    ($($name: ident),*) => {
        #[allow(non_snake_case)]
        impl<'a, Func, Return, $($name: Query),*> IntoFunction<($($name,)*)> for Func
        where
            Func:
                FnMut($($name),*) -> Return +
                FnMut($(<<$name as Query>::Fetch as Fetch>::Item),* ) -> Return +
                Send + Sync + 'static {
            fn into_function(mut self) -> Function {
                #[allow(unused_mut)]
                let mut borrows = Borrows::default();
                ($(<<$name as Query>::Fetch as Fetch>::borrow_check(&mut borrows),)*);

                Function {
                    exec: Box::new(move |g_state| {
                        unsafe {
                            let ($($name,)*) = g_state.query::<($($name,)*)>();

                            self($($name,)*);
                        }
                    }),
                    borrows
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
