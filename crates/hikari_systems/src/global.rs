use std::{any::TypeId, marker::PhantomPinned, pin::Pin};

use fxhash::FxHashMap;

use crate::{
    query::{Fetch, Query},
    State,
};

pub use crate::borrow::{Ref, RefMut, StateCell};

pub struct GlobalStateBuilder {
    g_state: UnsafeGlobalState,
}

impl GlobalStateBuilder {
    pub fn new() -> Self {
        Self {
            g_state: UnsafeGlobalState {
                state_list: Default::default(),
                _marker: PhantomPinned::default()
            },
        }
    }
    pub fn add_state<S: State>(mut self, state: S) -> Self {
        if let Some(_) = self
            .g_state
            .state_list
            .insert(TypeId::of::<S>(), StateCell::new(state))
        {
            panic!(
                "State of type {} has already been registered",
                std::any::type_name::<S>()
            );
        }

        self
    }
    pub fn build(self) -> GlobalState {
        GlobalState {
            inner: Box::pin(self.g_state),
        }
    }
}

unsafe impl Send for UnsafeGlobalState {}
unsafe impl Sync for UnsafeGlobalState {}

//Access to Internal state is not guaranteed to be thread safe, because of thread_unsafety feature
pub struct UnsafeGlobalState {
    state_list: FxHashMap<TypeId, StateCell>,
    _marker: PhantomPinned
}

impl UnsafeGlobalState {
    pub fn new() -> GlobalStateBuilder {
        GlobalStateBuilder::new()
    }
    pub unsafe fn get<S: State>(self: Pin<&Self>) -> Option<Ref<S>> {
        self.get_ref().state_list
            .get(&TypeId::of::<S>())
            .map(|cell| cell.borrow_cast())
    }
    pub unsafe fn get_mut<S: State>(self: Pin<&Self>) -> Option<RefMut<S>> {
        self.get_ref().state_list
            .get(&TypeId::of::<S>())
            .map(|cell| cell.borrow_cast_mut())
    }
    pub unsafe fn query<Q: Query>(self: Pin<&Self>) -> <<Q as Query>::Fetch as Fetch<'_>>::Item {
        Q::Fetch::get(self)
    }
}

pub struct GlobalState {
    inner: Pin<Box<UnsafeGlobalState>>,
}
impl GlobalState {
    pub fn new() -> GlobalStateBuilder {
        GlobalStateBuilder::new()
    }
    #[inline]
    pub fn raw(&self) -> Pin<&UnsafeGlobalState> {
        self.inner.as_ref()
    }
    #[inline]
    pub fn raw_mut(&mut self) -> Pin<&mut UnsafeGlobalState> {
        self.inner.as_mut()
    }
    #[inline]
    pub unsafe fn get<S: State>(&self) -> Option<Ref<S>> {
        self.raw().get() 
    }
    #[inline]
    pub unsafe fn get_mut<S: State>(&self) -> Option<RefMut<S>> {
        UnsafeGlobalState::get_mut(self.raw())
    }

}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::{
        GlobalState,
    };

    pub struct Renderer {
        x: i32,
    }

    pub struct Physics {
        y: bool,
    }
    #[test]
    fn speed() {
        let n = 1_000_000;

        let context = GlobalState::new()
            .add_state(Renderer { x: 0 })
            .add_state(Physics { y: false })
            .build();

        let now = Instant::now();
        //let x = tuple.deref();
        // let (a,b) = tuple;
        // let refs = (&*a, &*b);

        let mut sum: i32 = 0;
        for _ in 0..n {
            let phys = unsafe { context.get_mut::<Physics>().unwrap() };
            let mut renderer = unsafe { context.get_mut::<Renderer>().unwrap() };
            renderer.x = rand::random();

            sum = sum.wrapping_add(renderer.x + phys.y as i32);
        }
        println!("sum {} {:?}", sum, now.elapsed());
    }
}
