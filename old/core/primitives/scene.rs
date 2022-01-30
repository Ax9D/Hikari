use super::{identifier::Name, Transform};

pub use hecs::*;
pub struct Scene {
    inner: hecs::World,
}
pub type Entity = hecs::Entity;
pub type Ref<'a, T> = hecs::Ref<'a, T>;
pub type RefMut<'a, T> = hecs::RefMut<'a, T>;

//use std::ops::Deref;
// impl Deref for World {
//     type Target = hecs::World;

//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// use std::ops::DerefMut;
// impl DerefMut for World {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }

impl Scene {
    pub fn new() -> Self {
        Self {
            inner: hecs::World::default(),
        }
    }
    pub fn create_entity(&mut self, name: &str) -> Entity {
        self.create_entity_with_transform(name, Transform::default())
    }
    pub fn create_entity_with_transform(&mut self, name: &str, transform: Transform) -> Entity {
        let name = if name == "" {
            Name::default()
        } else {
            Name::new(name)
        };
        self.inner.spawn((transform, name))
    }

    pub fn delete_entity(&mut self, entity: hecs::Entity) {
        self.inner.despawn(entity).unwrap();
    }

    pub fn add_component<T: hecs::Component>(&mut self, entity: Entity, component: T) {
        self.inner.insert_one(entity, component).unwrap();
    }
    pub fn get_component<T: hecs::Component>(
        &mut self,
        entity: Entity,
    ) -> Result<Ref<T>, hecs::ComponentError> {
        self.inner.get::<T>(entity)
    }
    pub fn get_component_mut<T: hecs::Component>(
        &mut self,
        entity: Entity,
    ) -> Result<RefMut<T>, hecs::ComponentError> {
        self.inner.get_mut::<T>(entity)
    }

    pub fn remove_component<T: hecs::Component>(&mut self, entity: Entity) {
        self.inner.remove_one::<T>(entity).unwrap();
    }

    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<Q> {
        self.inner.query()
    }
}
