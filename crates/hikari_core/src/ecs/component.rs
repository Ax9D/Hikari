use hecs::{NoSuchEntity, ComponentError};

#[allow(unused_variables)]
pub trait Component: hecs::Component {
    fn on_added(world: &mut crate::World, entity: crate::Entity, component: Self) -> Result<(), NoSuchEntity> where Self: Sized {
        world.raw_mut().insert_one(entity, component)
    }
    fn on_removed(world: &mut crate::World, entity: crate::Entity) -> Result<Self, ComponentError> where Self: Sized  {
        world.raw_mut().remove_one(entity)
    }
}

impl<C: hecs::Component> Component for C {}