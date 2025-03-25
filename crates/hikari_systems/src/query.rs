use std::{marker::PhantomData, pin::Pin};

use fxhash::FxHashMap;

use crate::{global::UnsafeGlobalState, State};
#[allow(dead_code)]
pub(crate) struct Type {
    pub name: &'static str,
    pub id: std::any::TypeId,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum BorrowKind {
    Shared,
    Mutable,
}
#[allow(dead_code)]
pub(crate) struct Borrow {
    pub ty: Type,
    pub kind: BorrowKind,
}
#[derive(Default)]
pub struct Borrows {
    map: FxHashMap<std::any::TypeId, Borrow>,
}

impl Borrows {
    fn borrow_int<T: 'static>(&mut self, kind: BorrowKind) {
        let id = std::any::TypeId::of::<T>();
        let name = std::any::type_name::<T>();
        if let Some(borrow) = self.map.get(&id) {
            let prev_kind = borrow.kind;
            let allowed = match kind {
                BorrowKind::Shared => prev_kind == BorrowKind::Shared,
                BorrowKind::Mutable => false,
            };

            if !allowed {
                panic!(
                    "Cannot borrow {} as {:?} as it was previosly borrowed as {:?}",
                    name, kind, prev_kind
                );
            }
        } else {
            self.map.insert(
                id,
                Borrow {
                    ty: Type { name, id },
                    kind,
                },
            );
        }
    }
    fn borrow<T: 'static>(&mut self) {
        self.borrow_int::<T>(BorrowKind::Shared)
    }
    fn borrow_mut<T: 'static>(&mut self) {
        self.borrow_int::<T>(BorrowKind::Mutable)
    }
}

pub trait Query {
    type Fetch: for<'a> Fetch<'a>;
}

pub unsafe trait Fetch<'a>: Sized {
    type Item;

    fn get(g_state: Pin<&'a UnsafeGlobalState>) -> Self::Item;

    fn borrow_check(borrows: &mut Borrows);
}

pub struct RefFetch<T> {
    _phantom: PhantomData<T>,
}

impl<'a, S: State> Query for &'a S {
    type Fetch = RefFetch<S>;
}

unsafe impl<'a, S: State> Fetch<'a> for RefFetch<S> {
    type Item = &'a S;

    fn get(g_state: Pin<&'a UnsafeGlobalState>) -> Self::Item {
        let result = unsafe {
            g_state
                .get_unchecked::<S>()
        };
        match result {
            Some(state) => state,
            None => panic!("No state of type: {}", std::any::type_name::<S>()),
        }
    }

    fn borrow_check(borrows: &mut Borrows) {
        borrows.borrow::<S>()
    }
}
pub struct RefMutFetch<T> {
    _phantom: PhantomData<T>,
}
impl<'a, S: State> Query for &'a mut S {
    type Fetch = RefMutFetch<S>;
}

unsafe impl<'a, S: State> Fetch<'a> for RefMutFetch<S> {
    type Item = &'a mut S;

    fn get(g_state: Pin<&'a UnsafeGlobalState>) -> Self::Item {
        let result = unsafe {
            UnsafeGlobalState::get_unchecked_mut::<S>(g_state)
        };

        match result {
            Some(state) => state,
            None => panic!("No state of type: {}", std::any::type_name::<S>()),
        }
    }

    fn borrow_check(borrows: &mut Borrows) {
        borrows.borrow_mut::<S>()
    }
}

impl<'a, S: State> Query for Option<&'a S> {
    type Fetch = MaybeRefFetch<S>;
}
pub struct MaybeRefFetch<T> {
    _phantom: PhantomData<T>,
}
unsafe impl<'a, S: State> Fetch<'a> for MaybeRefFetch<S> {
    type Item = Option<&'a S>;

    fn get(g_state: Pin<&'a UnsafeGlobalState>) -> Self::Item {
        unsafe { g_state.get_unchecked::<S>() }
    }

    fn borrow_check(borrows: &mut Borrows) {
        borrows.borrow::<S>()
    }
}

impl<'a, S: State> Query for Option<&'a mut S> {
    type Fetch = MaybeRefMutFetch<S>;
}
pub struct MaybeRefMutFetch<T> {
    _phantom: PhantomData<T>,
}
unsafe impl<'a, S: State> Fetch<'a> for MaybeRefMutFetch<S> {
    type Item = Option<&'a mut S>;

    fn get(g_state: Pin<&'a UnsafeGlobalState>) -> Self::Item {
        unsafe { g_state.get_unchecked_mut::<S>() }
    }

    fn borrow_check(borrows: &mut Borrows) {
        borrows.borrow_mut::<S>()
    }
}

impl Query for () {
    type Fetch = ();
}
unsafe impl<'a> Fetch<'a> for () {
    type Item = ();

    fn get(_: Pin<&'a UnsafeGlobalState>) -> Self::Item {
        ()
    }
    fn borrow_check(_: &mut Borrows) {}
}

macro_rules! impl_query {
    ($($name: ident),*) => {
        impl<$($name: Query),*> Query for ($($name,)*) {
            type Fetch = ($($name::Fetch,)*);
        }

        unsafe impl<'a, $($name: Fetch<'a>),*> Fetch<'a> for ($($name,)*) {
            type Item = ($($name::Item,)*);

            fn get(g_state: Pin<&'a UnsafeGlobalState>) -> Self::Item {
                ($($name::get(g_state),)*)
            }
            fn borrow_check(borrows: &mut Borrows) {
                ($($name::borrow_check(borrows),)*);
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
