use std::any::TypeId;

use crate::{CloneComponent, Component, Registry};

pub type Entity = hecs::Entity;

use hecs::DynamicBundle;
pub use hecs::{
    CommandBuffer, ComponentError, ComponentRef, EntityRef, MissingComponent, NoSuchEntity, Query,
    QueryBorrow, QueryItem, QueryMut, QueryOne, Ref, RefMut, With, Without,
};
use hikari_math::Transform;
use uuid::Uuid;
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
        self.world.spawn((Transform::default(), Uuid::new_v4()))
    }
    #[inline]
    pub fn uuid(&self, entity: Entity) -> Uuid {
        *self.get_component::<&Uuid>(entity).unwrap()
    }
    pub fn create_entity_with(&mut self, components: impl DynamicBundle) -> Entity {
        let mut has_uuid = false;
        let mut has_transform = false;
        components.with_ids(|ids| {
            for ty in ids {
                if &TypeId::of::<Transform>() == ty {
                    has_transform = true;
                } else if &TypeId::of::<Uuid>() == ty {
                    has_uuid = true;
                }
            }
        });
        let entity = self.world.spawn(components);

        if !has_transform {
            self.add_component(entity, Transform::default()).unwrap();
        }
        if !has_uuid {
            self.add_component(entity, Uuid::new_v4()).unwrap();
        }
        entity
    }
    pub fn create_entity_at(&mut self, handle: Entity, components: impl DynamicBundle) {
        let mut has_uuid = false;
        let mut has_transform = false;
        components.with_ids(|ids| {
            for ty in ids {
                if &TypeId::of::<Transform>() == ty {
                    has_transform = true;
                } else if &TypeId::of::<Uuid>() == ty {
                    has_uuid = true;
                }
            }
        });
        self.world.spawn_at(handle, components);

        if !has_transform {
            self.add_component(handle, Transform::default()).unwrap();
        }
        if !has_uuid {
            self.add_component(handle, Uuid::new_v4()).unwrap();
        }
    }
    #[inline]
    pub fn remove_entity(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.world.despawn(entity)
    }
    pub fn clone_entity(
        &mut self,
        entity: Entity,
        registry: &Registry,
    ) -> Result<EntityBuilder, NoSuchEntity> {
        let entity = self.entity(entity)?;
        Ok(registry.clone_entity(entity))
    }
    pub fn duplicate_entity(
        &mut self,
        entity: Entity,
        registry: &Registry,
    ) -> Result<Entity, NoSuchEntity> {
        let mut builder = self.clone_entity(entity, registry)?;

        //Replace with new uuid
        builder.add(Uuid::new_v4());

        let dup_entity = self.world.spawn(builder.build());

        Ok(dup_entity)
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
    pub fn clone_into(&self, registry: &Registry, dst: &mut World) {
        for entity in self.entities() {
            let mut builder = registry.clone_entity(entity);
            let entity = entity.entity();
            dst.create_entity_at(entity, builder.build());
        }
    }
    pub fn clone(&self, registry: &Registry) -> World {
        let mut dst = World::new();
        self.clone_into(registry, &mut dst);

        dst
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
    pub fn get_component<'a, C: ComponentRef<'a>>(
        &'a self,
        entity: Entity,
    ) -> Result<C::Ref, ComponentError> {
        self.world.get::<C>(entity)
    }
    #[inline]
    pub fn has_component<C: Component>(&self, entity: Entity) -> bool {
        self.world.get::<&C>(entity).is_ok()
    }
    pub fn clone_component_untyped(
        &self,
        type_id: TypeId,
        src: Entity,
        dst: Entity,
        dst_world: &mut World,
        registry: &Registry,
    ) -> Result<(), NoSuchEntity> {
        registry.clone_component_untyped(type_id, self.entity(src)?, dst, dst_world)
    }
    pub fn clone_component<C: CloneComponent>(
        &self,
        src: Entity,
        dst: Entity,
        dst_world: &mut World,
        registry: &Registry,
    ) -> Result<(), NoSuchEntity> {
        registry.clone_component::<C>(self.entity(src)?, dst, dst_world)
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

pub type EntityBuilder = hecs::EntityBuilder;
