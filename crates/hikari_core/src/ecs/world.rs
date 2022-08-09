use crate::Component;

pub type Entity = hecs::Entity;

use hecs::DynamicBundle;
pub use hecs::{
    CommandBuffer, ComponentError, EntityRef, MissingComponent, NoSuchEntity, Query, QueryBorrow,
    QueryItem, QueryMut, QueryOne, Ref, RefMut, With, Without,
};
use hikari_math::Transform;
pub struct World {
    world: hecs::World,
}
impl World {
    pub fn new() -> Self {
        Self {
            world: hecs::World::default(),
        }
    }
    #[inline]
    pub fn create_entity(&mut self) -> Entity {
        self.world.spawn((Transform::default(),))
    }
    #[inline]
    pub fn create_entity_at(&mut self, handle: Entity, components: impl DynamicBundle) {
        self.world.spawn_at(handle, components);
    }
    #[inline]
    pub fn remove_entity(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.world.despawn(entity)
    }
    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.world.contains(entity)
    }
    #[inline]
    pub fn entity(&self, entity: Entity) -> Result<EntityRef, NoSuchEntity> {
        self.world.entity(entity)
    }
    #[inline]
    pub fn entities(&self) -> hecs::Iter<'_> {
        self.world.iter()
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.world.len() as usize
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.world.is_empty()
    }
    #[inline]
    pub fn clear(&mut self) {
        self.world.clear()
    }
    #[inline]
    pub fn add_component(
        &mut self,
        entity: Entity,
        component: impl Component,
    ) -> Result<(), NoSuchEntity> {
        self.world.insert_one(entity, component)
    }
    #[inline]
    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Result<C, ComponentError> {
        self.world.remove_one::<C>(entity)
    }
    #[inline]
    pub fn get_component<C: Component>(
        &self,
        entity: Entity,
    ) -> Result<Ref<'_, C>, ComponentError> {
        self.world.get(entity)
    }
    #[inline]
    pub fn get_component_mut<C: Component>(
        &self,
        entity: Entity,
    ) -> Result<RefMut<'_, C>, ComponentError> {
        self.world.get_mut(entity)
    }
    #[inline]
    pub fn run_query<Q: Query>(&self, mut f: impl FnMut(Entity, QueryItem<Q>)) {
        for (entity, item) in self.query::<Q>().iter() {
            (f)(entity, item);
        }
    }
    #[inline]
    pub fn run_query_mut<Q: Query>(&mut self, mut f: impl FnMut(Entity, QueryItem<Q>)) {
        for (entity, item) in self.query_mut::<Q>() {
            (f)(entity, item);
        }
    }
    #[inline]
    pub fn query<Q: Query>(&self) -> QueryBorrow<'_, Q> {
        self.world.query::<Q>()
    }
    #[inline]
    pub fn query_mut<Q: Query>(&mut self) -> QueryMut<'_, Q> {
        self.world.query_mut::<Q>()
    }
    #[inline]
    pub fn query_one<Q: Query>(&self, entity: Entity) -> Result<QueryOne<'_, Q>, NoSuchEntity> {
        self.world.query_one(entity)
    }
    #[inline]
    pub fn execute_commands(&mut self, mut cmd: CommandBuffer) {
        cmd.run_on(&mut self.world);
    }
}
