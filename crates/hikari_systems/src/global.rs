use std::{any::TypeId, marker::PhantomPinned, pin::Pin};

use fxhash::FxHashMap;

use crate::{
    query::{Fetch, Query},
    State,
};

use crate::borrow::{Ref, RefMut, StateCell};

pub struct StateBuilder {
    state_list: FxHashMap<TypeId, StateCell>,
}

impl StateBuilder {
    pub fn new() -> Self {
        Self {
            state_list: Default::default(),
        }
    }
    pub fn add_state<S: State>(&mut self, state: S) -> &mut Self {
        if self
            .state_list
            .insert(TypeId::of::<S>(), StateCell::new(state))
            .is_some()
        {
            panic!(
                "State of type {} has already been registered",
                std::any::type_name::<S>()
            );
        }

        self
    }
    pub fn get<S: State>(&self) -> Ref<S> {
        self.state_list
            .get(&TypeId::of::<S>())
            .map(|cell| cell.borrow_cast::<S>())
            .unwrap()
    }
    pub fn get_mut<S: State>(&self) -> RefMut<S> {
        self.state_list
            .get(&TypeId::of::<S>())
            .map(|cell| cell.borrow_cast_mut::<S>())
            .unwrap()
    }
    pub fn build(self) -> GlobalState {
        GlobalState {
            inner: Box::pin(UnsafeGlobalState {
                state_list: self.state_list,
                _marker: PhantomPinned::default(),
            }),
        }
    }
}

unsafe impl Send for UnsafeGlobalState {}
unsafe impl Sync for UnsafeGlobalState {}

#[derive(Default)]
pub struct UnsafeGlobalState {
    state_list: FxHashMap<TypeId, StateCell>,
    _marker: PhantomPinned,
}

impl UnsafeGlobalState {
    pub fn get<S: State>(self: Pin<&Self>) -> Option<Ref<S>> {
        self.get_ref()
            .state_list
            .get(&TypeId::of::<S>())
            .map(|cell| cell.borrow_cast())
    }
    pub fn get_mut<S: State>(self: Pin<&Self>) -> Option<RefMut<S>> {
        self.get_ref()
            .state_list
            .get(&TypeId::of::<S>())
            .map(|cell| cell.borrow_cast_mut())
    }
    pub(crate) unsafe fn get_unchecked<'a, S: State>(self: Pin<&'a Self>) -> Option<&'a S> {
        self.get_ref()
            .state_list
            .get(&TypeId::of::<S>())
            .map(|cell| cell.borrow_cast_unchecked())
    }
    pub(crate) unsafe fn get_unchecked_mut<'a, S: State>(self: Pin<&'a Self>) -> Option<&'a mut S> {
        self.get_ref()
            .state_list
            .get(&TypeId::of::<S>())
            .map(|cell| cell.borrow_cast_unchecked_mut())
    }
    pub(crate) unsafe fn query<Q: Query>(self: Pin<&Self>) -> <<Q as Query>::Fetch as Fetch<'_>>::Item {
        Q::Fetch::get(self)
    }
}
pub struct UniqueGlobalState {}
pub struct GlobalState {
    inner: Pin<Box<UnsafeGlobalState>>,
}
impl GlobalState {
    #[inline]
    pub fn raw(&self) -> Pin<&UnsafeGlobalState> {
        self.inner.as_ref()
    }
    #[inline]
    pub fn raw_mut(&mut self) -> Pin<&mut UnsafeGlobalState> {
        self.inner.as_mut()
    }
    #[inline]
    pub fn get<S: State>(&self) -> Option<Ref<S>> {
        self.raw().get()
    }
    #[inline]
    pub fn get_mut<S: State>(&self) -> Option<RefMut<S>> {
        UnsafeGlobalState::get_mut(self.raw())
    }

    #[inline]
    pub fn run(&mut self, function: &mut crate::Function) {
        unsafe {
            function.run(self.raw());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::StateBuilder;

    pub struct Renderer {
        x: i32,
    }

    pub struct Physics {
        y: bool,
    }
    #[test]
    fn speed() {
        let n = 1_000_000;

        let mut global = StateBuilder::new();
        global.add_state(Renderer { x: 0 });
        global.add_state(Physics { y: false });
        let global = global.build();

        let now = Instant::now();
        //let x = tuple.deref();
        // let (a,b) = tuple;
        // let refs = (&*a, &*b);

        let mut sum: i32 = 0;
        for _ in 0..n {
            let phys = global.get_mut::<Physics>().unwrap();
            let mut renderer = global.get_mut::<Renderer>().unwrap();
            renderer.x = rand::random();

            sum = sum.wrapping_add(renderer.x + phys.y as i32);
        }
        println!("sum {} {:?}", sum, now.elapsed());
    }
}
