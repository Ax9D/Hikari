use std::marker::PhantomData;

use crate::{
    borrow::{Ref, RefMut},
    global::UnsafeGlobalState,
    State,
};

pub trait Query {
    type Fetch: for<'a> Fetch<'a>;
}

pub unsafe trait Fetch<'a>: Sized {
    type Item;

    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item;
}

pub struct RefFetch<T> {
    _phantom: PhantomData<T>
}

impl<'a, S:State> Query for Ref<'a, S> {
    type Fetch = RefFetch<S>;
}

unsafe impl<'a, S: State> Fetch<'a> for RefFetch<S> {
    type Item = Ref<'a, S>;

    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
        unsafe{ g_state.get::<S>().unwrap() }
    }
}

pub struct RefMutFetch<T> {
    _phantom: PhantomData<T>
}
impl<'a, S:State> Query for RefMut<'a, S> {
    type Fetch = RefFetch<S>;
}

unsafe impl<'a, S: State> Fetch<'a> for RefMutFetch<S> {
    type Item = RefMut<'a, S>;

    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
        unsafe{ g_state.get_mut::<S>().unwrap() }
    }
}

impl Query for () {
    type Fetch = ();
}

macro_rules! impl_query {
    ($($name: ident),*) => {
        impl<$($name: Query),*> Query for ($($name,)*) {
            type Fetch = ($($name::Fetch,)*);
        }
 

        unsafe impl<'a, $($name: Fetch<'a>),*> Fetch<'a> for ($($name,)*) {
            type Item = ($($name::Item,)*);

            fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
                ($($name::get(g_state),)*)
            }
        }
    }
}
impl_query!(A);
impl_query!(A, B);
impl_query!(A, B, C);
impl_query!(A, B, C, D);
impl_query!(A, B, C, D, E);
impl_query!(A, B, C, D, E, F);
impl_query!(A, B, C, D, E, F, G);
impl_query!(A, B, C, D, E, F, G, H);

unsafe impl<'a> Fetch<'a> for () {
    type Item = ();

    #[allow(unused_variables)]
    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
        ()
    }
}
