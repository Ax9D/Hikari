use std::any::TypeId;

use fxhash::FxHashMap;

use crate::{State, borrow::{Ref, RefMut, StateCell}};

pub struct ContextBuilder {
    context: Context,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            context: Context {
                state_list: Default::default(),
            },
        }
    }
    pub fn add_state<S: State>(mut self, state: S) -> Self{
        if let Some(_) = self
            .context
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
    pub fn build(self) -> Context {
        self.context
    }
}
pub struct Context {
    state_list: FxHashMap<TypeId, StateCell>,
}

impl Context {
    pub fn new() -> ContextBuilder {
        ContextBuilder::new()
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

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::Context;

    pub struct Renderer {
        x: i32
    }

    pub struct Physics {
        y: bool
    }
    #[test]
    fn speed() {
        let n = 1_000_000;

        let context = Context::new()
        .add_state(Renderer {
            x: 0
        })
        .add_state(Physics {
            y: false
        })
        .build();

        let now = Instant::now();

        let mut sum = 0;
        for _ in 0..n {
            let phys = context.get_mut::<Physics>().unwrap();
            let mut renderer = context.get_mut::<Renderer>().unwrap();
            renderer.x=rand::random();

            sum+= renderer.x + phys.y as i32;
        }
        println!("sum {} {:?}", sum, now.elapsed());
    }
}