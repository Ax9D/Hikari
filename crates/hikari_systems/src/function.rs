use crate::query::Borrows;
use crate::query::Query;

pub type VoidFunction = Function<()>;

#[allow(dead_code)]
pub struct Function<R> {
    borrows: Borrows,
    exec: Box<dyn FnMut(Pin<&UnsafeGlobalState>) -> R + 'static>,
}
impl<R> Function<R> {
    pub unsafe fn from_raw(exec: Box<dyn FnMut(Pin<&UnsafeGlobalState>) -> R + 'static>) -> Self {
        Self {
            borrows: Borrows::default(),
            exec,
        }
    }
    #[inline]
    pub(crate) unsafe fn run(&mut self, g_state: Pin<&UnsafeGlobalState>) -> R {
        (self.exec)(g_state)
    }
}
pub trait IntoFunction<Params, R> {
    fn into_function(self) -> Function<R>;
}

use std::pin::Pin;

use crate::global::UnsafeGlobalState;
use crate::query::Fetch;

macro_rules! impl_into_function {
    ($($name: ident),*) => {
        #[allow(non_snake_case)]
        impl<'a, Func, Return, $($name: Query),*> IntoFunction<($($name,)*), Return> for Func
        where
            Func:
                FnMut($($name),*) -> Return +
                FnMut($(<<$name as Query>::Fetch as Fetch>::Item),* ) -> Return +
                Send + Sync + 'static {
            fn into_function(mut self) -> Function<Return> {
                #[allow(unused_mut)]
                let mut borrows = Borrows::default();
                ($(<<$name as Query>::Fetch as Fetch>::borrow_check(&mut borrows),)*);

                Function {
                    exec: Box::new(move |g_state| {
                        unsafe {
                            let ($($name,)*) = g_state.query::<($($name,)*)>();

                            self($($name,)*)
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
