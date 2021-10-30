use std::marker::PhantomData;

use crate::{State, atomic_borrow::{Ref, RefMut}, global::UnsafeGlobalState};

trait Fetch<'a>: Sized {
    type Item: DerefTuple<'a>;
    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item;
}

struct FetchRead<T> {
    _phantom: PhantomData<T>
}

struct FetchWrite<T> {
    _phantom: PhantomData<T>
}


// impl<'a, S: State> Fetch<'a> for FetchRead<S> {
//     type Item = Ref<'a, S>;

//     fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
//         g_state.get::<S>().unwrap()
//     }
// }

// impl<'a, S: State> Fetch<'a> for FetchWrite<S> {
//     type Item = RefMut<'a, S>;

//     fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
//         g_state.get_mut::<S>().unwrap()
//     }
// }


impl<'a, S: State> Fetch<'a> for &'a S {
    type Item = Ref<'a, S>;

    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
        g_state.get::<S>().unwrap()
    }
}

impl<'a,A:State, B: State> Fetch<'a> for (&'a A, &'a B) {
    type Item = (Ref<'a, A>, Ref<'a, B>);

    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
        (g_state.get::<A>().unwrap(), g_state.get::<B>().unwrap())
    }
}

impl <'a, S: State> Fetch<'a> for &'a mut S {
    type Item = RefMut<'a, S>;

    fn get(g_state: &'a UnsafeGlobalState) -> Self::Item {
        g_state.get_mut::<S>().unwrap()
    }
}
trait DerefTuple<'a> {
    type Target;

    fn deref(&'a mut self) -> Self::Target;
}
impl <'a, A: State> DerefTuple<'a> for Ref<'a, A> {
    type Target = &'a A;

    fn deref(&'a mut self) -> Self::Target {
        let a = self;
        &* a
    }
}
impl<'a, A: State, B: State> DerefTuple<'a> for (Ref<'a, A>, Ref<'a, B>) {
    type Target = (&'a A, &'a B);

    fn deref(&'a mut self) -> Self::Target {
        let (a, b) = self;
        (&*a, &*b)
    }
}

impl<'a, A: State, B: State> DerefTuple<'a> for (Ref<'a, A>, RefMut<'a, B>) {
    type Target = (&'a A, &'a mut B);

    fn deref(&'a mut self) -> Self::Target {
        let (a, b) = self;
        (&*a, &mut *b)
    }
}

impl <'a, A: State> DerefTuple<'a> for RefMut<'a, A> {
    type Target = &'a mut A;

    fn deref(&'a mut self) -> Self::Target {
        let a = self;
        &mut *a
    }
}