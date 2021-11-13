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
    _phantom: PhantomData<T>,
}

impl<'a, S: State> Query for Ref<'a, S> {
    type Fetch = RefFetch<S>;
}

unsafe impl<'a, S: State> Fetch<'a> for RefFetch<S> {
    type Item = Ref<'a, S>;

    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
        unsafe { g_state.get::<S>().expect(&format!("No state of type: {}", std::any::type_name::<S>())) }
    }
}
pub struct RefMutFetch<T> {
    _phantom: PhantomData<T>,
}
impl<'a, S: State> Query for RefMut<'a, S> {
    type Fetch = RefMutFetch<S>;
}

unsafe impl<'a, S: State> Fetch<'a> for RefMutFetch<S> {
    type Item = RefMut<'a, S>;

    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
        unsafe { g_state.get_mut::<S>().expect(&format!("No state of type: {}", std::any::type_name::<S>())) }
    }
}

impl<'a, S:State> Query for Option<Ref<'a, S>> {
    type Fetch = MaybeRefFetch<S>;
}
pub struct MaybeRefFetch<T> {
    _phantom: PhantomData<T>
}
unsafe impl<'a, S: State> Fetch<'a> for MaybeRefFetch<S> {
    type Item = Option<Ref<'a, S>>;

    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
        unsafe { g_state.get::<S>() }
    }
}

impl<'a, S:State> Query for Option<RefMut<'a, S>> {
    type Fetch = MaybeRefMutFetch<S>;
}
pub struct MaybeRefMutFetch<T> {
    _phantom: PhantomData<T>
}
unsafe impl<'a, S: State> Fetch<'a> for MaybeRefMutFetch<S> {
    type Item = Option<RefMut<'a, S>>;

    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
        unsafe { g_state.get_mut::<S>() }
    }
}


impl Query for () {
    type Fetch = ();
}
unsafe impl<'a> Fetch<'a> for () {
    type Item = ();

    #[allow(unused_variables)]
    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
        ()
    }
}

impl<'a> Query for &'a UnsafeGlobalState {
    type Fetch = UnsafeGlobalFetch;
} 
pub struct UnsafeGlobalFetch;

unsafe impl<'a> Fetch<'a> for UnsafeGlobalFetch {
    type Item = &'a UnsafeGlobalState;

    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
        g_state
    }
    
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