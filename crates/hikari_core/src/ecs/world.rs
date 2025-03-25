use std::any::TypeId;

use crate::{CloneComponent, Component, Registry, EntityId};

pub type Entity = hecs::Entity;

use hecs::DynamicBundle;
pub use hecs::{
    CommandBuffer, ComponentError, ComponentRef, EntityRef, MissingComponent, NoSuchEntity, Query,
    QueryBorrow, QueryItem, QueryMut, QueryOne, Ref, RefMut, With, Without,
    PreparedQueryBorrow, PreparedQueryIter, PreparedView,
};
use hikari_asset::Asset;
use hikari_math::Transform;
use uuid::Uuid;

#[derive(type_uuid::TypeUuid)]
#[uuid = "c2103fb7-13fe-4f18-84a5-f6075ce7206a"]
pub struct World {
    inner: hecs::World,
}

impl World {
    pub fn new() -> Self {
        Self {
            inner: hecs::World::default(),
        }
    }
    #[inline]
    pub fn raw(&self) -> &hecs::World {
        &self.inner
    }
    #[inline]
    pub fn raw_mut(&mut self) -> &mut hecs::World {
        &mut self.inner
    }
    #[inline]
    pub fn create_entity(&mut self) -> Entity {
        self.inner.spawn((Transform::default(), EntityId::default()))
    }
    #[inline]
    pub fn create_entity_with_name(&mut self, name: impl AsRef<str>) -> Entity {
        self.inner.spawn((Transform::default(), EntityId::new_with_name(name)))
    }
    #[inline]
    pub fn entity_id(&self, entity: Entity) -> Result<Ref<EntityId>, ComponentError> {
        self.get_component::<&EntityId>(entity)
    }
    #[inline]
    pub fn entity_uuid(&self, entity: Entity) -> Result<Uuid, ComponentError> {
        match self.entity_id(entity) {
            Ok(id) => Ok(id.uuid),
            Err(err) => Err(err)
        }
    }
    pub fn create_entity_with(&mut self, components: impl DynamicBundle) -> Entity {
        let mut has_id = false;
        let mut has_transform = false;
        components.with_ids(|ids| {
            for ty in ids {
                if &TypeId::of::<Transform>() == ty {
                    has_transform = true;
                } else if &TypeId::of::<EntityId>() == ty {
                    has_id = true;
                }
            }
        });
        let entity = self.inner.spawn(components);

        if !has_transform {
            self.add_component(entity, Transform::default()).unwrap();
        }
        if !has_id {
            self.add_component(entity, EntityId::new()).unwrap();
        }
        entity
    }
    pub fn create_entity_at(&mut self, handle: Entity, components: impl DynamicBundle) {
        let mut has_id = false;
        let mut has_transform = false;
        components.with_ids(|ids| {
            for ty in ids {
                if &TypeId::of::<Transform>() == ty {
                    has_transform = true;
                } else if &TypeId::of::<EntityId>() == ty {
                    has_id = true;
                }
            }
        });
        self.inner.spawn_at(handle, components);

        if !has_transform {
            self.add_component(handle, Transform::default()).unwrap();
        }
        if !has_id {
            self.add_component(handle, EntityId::new()).unwrap();
        }
    }
    #[inline]
    pub fn remove_entity(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.inner.despawn(entity)
    }
    pub fn clone_entity(
        &mut self,
        entity: Entity,
        registry: &Registry,
    ) -> Result<EntityBuilder, NoSuchEntity> {
        hikari_dev::profile_function!();
        let entity = self.entity(entity)?;
        Ok(registry.clone_entity(entity))
    }
    pub fn duplicate_entity(
        &mut self,
        entity: Entity,
        registry: &Registry,
    ) -> Result<Entity, NoSuchEntity> {
        let mut builder = self.clone_entity(entity, registry)?;

        let id = builder.get_mut::<&mut EntityId>().unwrap();

        //Change uuid
        id.uuid = Uuid::new_v4();

        let dup_entity = self.inner.spawn(builder.build());

        Ok(dup_entity)
    }
    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.inner.contains(entity)
    }
    #[inline]
    pub fn entity(&self, entity: Entity) -> Result<EntityRef, NoSuchEntity> {
        self.inner.entity(entity)
    }
    #[inline]
    pub fn entities(&self) -> hecs::Iter<'_> {
        self.inner.iter()
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len() as usize
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear()
    }
    pub fn clone_into(&self, registry: &Registry, dst: &mut World) {
        hikari_dev::profile_function!();
        for entity in self.entities() {
            let mut builder = registry.clone_entity(entity);
            let entity = entity.entity();
            dst.create_entity_at(entity, builder.build());
        }
    }
    pub fn clone(&self, registry: &Registry) -> World {
        hikari_dev::profile_function!();
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
        Component::on_added(self, entity, component)
    }
    #[inline]
    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Result<C, ComponentError> {
        Component::on_removed(self, entity)
    }
    #[inline]
    pub fn get_component<'a, C: ComponentRef<'a>>(
        &'a self,
        entity: Entity,
    ) -> Result<C::Ref, ComponentError> {
        self.inner.get::<C>(entity)
    }
    #[inline]
    pub fn has_component<C: Component>(&self, entity: Entity) -> bool {
        self.get_component::<&C>(entity).is_ok()
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
        self.inner.query::<Q>()
    }
    #[inline]
    pub fn query_mut<Q: Query>(&mut self) -> QueryMut<'_, Q> {
        self.inner.query_mut::<Q>()
    }
    #[inline]
    pub fn query_one<Q: Query>(&self, entity: Entity) -> Result<QueryOne<'_, Q>, NoSuchEntity> {
        self.inner.query_one(entity)
    }
    #[inline]
    pub fn execute_commands(&mut self, mut cmd: CommandBuffer) {
        cmd.run_on(&mut self.inner);
    }
}

pub type EntityBuilder = hecs::EntityBuilder;


pub struct PreparedQuery<Q: Query> {
    inner: hecs::PreparedQuery<Q>
}

impl<Q: Query> PreparedQuery<Q> {
    /// Create a prepared query which is not yet attached to any world
    pub fn new() -> Self {
        Self {
            inner: hecs::PreparedQuery::new()
        }
    }

    /// Query `world`, using dynamic borrow checking
    ///
    /// This will panic if it would violate an existing unique reference
    /// or construct an invalid unique reference.
    pub fn query<'q>(&'q mut self, world: &'q World) -> PreparedQueryBorrow<'q, Q> {
        self.inner.query(&world.inner)
    }

    /// Query a uniquely borrowed world
    ///
    /// Avoids the cost of the dynamic borrow checking performed by [`query`][Self::query].
    pub fn query_mut<'q>(&'q mut self, world: &'q mut World) -> PreparedQueryIter<'q, Q> {
        self.inner.query_mut(&mut world.inner)
    }

    /// Provide random access to query results for a uniquely borrow world
    pub fn view_mut<'q>(&'q mut self, world: &'q mut World) -> PreparedView<'q, Q> {
        self.inner.view_mut(&mut world.inner)
    }
}

impl Drop for World {
    fn drop(&mut self) {
        log::debug!("Dropping world");
    }
}

impl Asset for World {
    type Settings = ();
}