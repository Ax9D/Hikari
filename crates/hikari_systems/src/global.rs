use std::{any::TypeId};

use fxhash::FxHashMap;

use crate::{State, atomic_borrow::{Ref, RefMut, StateCell}};

pub struct GlobalStateBuilder {
    g_state: UnsafeGlobalState,
}

impl GlobalStateBuilder {
    pub fn new() -> Self {
        Self {
            g_state: UnsafeGlobalState {
                state_list: Default::default(),
            },
        }
    }
    pub fn add_state<S: State>(mut self, state: S) -> Self{
        if let Some(_) = self
            .g_state
            .state_list
            .insert(TypeId::of::<S>(), StateCell::new(state))
        {
            panic!(
                "State {} has already been registered",
                std::any::type_name::<S>()
            );
        }

        self
    }
    pub fn build(self) -> GlobalState {
        GlobalState {
            inner: self.g_state
        }
    }
}

unsafe impl Send for UnsafeGlobalState {}
unsafe impl Sync for UnsafeGlobalState {}

pub(crate) struct UnsafeGlobalState {
    state_list: FxHashMap<TypeId, StateCell>,
}

impl UnsafeGlobalState {
    pub fn new() -> GlobalStateBuilder {
        GlobalStateBuilder::new()
    }
    #[inline]
    pub fn get<S: State>(&self) -> Option<Ref<S>> {
        self.state_list.get(&TypeId::of::<S>()).map(|cell|cell.borrow_cast())
    }
    #[inline]
    pub fn get_mut<S: State>(&self) -> Option<RefMut<S>> {
        self.state_list.get(&TypeId::of::<S>()).map(|cell|cell.borrow_cast_mut())
    }
}

pub struct GlobalState {
    pub(crate) inner: UnsafeGlobalState
}
impl GlobalState {
    pub fn new() -> GlobalStateBuilder {
        GlobalStateBuilder::new()
    }
    #[inline]
    pub fn get<S: State>(&self) -> Option<Ref<S>> {
        self.inner.get()
    }
    #[inline]
    pub fn get_mut<S: State>(&self) -> Option<RefMut<S>> {
        self.inner.get_mut()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::GlobalState;

    pub struct Renderer {
        x: i32
    }

    pub struct Physics {
        y: bool
    }
    #[test]
    fn speed() {
        let n = 1_000_000;

        let context = GlobalState::new()
        .add_state(Renderer {
            x: 0
        })
        .add_state(Physics {
            y: false
        })
        .build();

        let now = Instant::now();

        let mut sum: i32 = 0;
        for _ in 0..n {
            let phys = context.get_mut::<Physics>().unwrap();
            let mut renderer = context.get_mut::<Renderer>().unwrap();
            renderer.x=rand::random();

            sum= sum.wrapping_add(renderer.x + phys.y as i32);
        }
        println!("sum {} {:?}", sum, now.elapsed());
    }
}