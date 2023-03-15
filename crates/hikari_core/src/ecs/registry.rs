use hecs::NoSuchEntity;
use std::{any::TypeId, collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::{Component, Entity, EntityBuilder, EntityRef, World};

pub trait CloneComponent: Component + Clone {}

impl<C: Component + Clone> CloneComponent for C {}

#[derive(Clone)]
pub struct CloneFn {
    builder: fn(EntityRef<'_>, &mut EntityBuilder),
    world: fn(EntityRef<'_>, Entity, &mut World) -> Result<(), NoSuchEntity>,
}
impl CloneFn {
    pub fn into_builder(&self, entity: EntityRef<'_>, dst: &mut EntityBuilder) {
        (self.builder)(entity, dst)
    }
    pub fn into_world(
        &self,
        src: EntityRef<'_>,
        dst: Entity,
        dst_world: &mut World,
    ) -> Result<(), NoSuchEntity> {
        (self.world)(src, dst, dst_world)
    }
}

pub(crate) struct RegistryInner {
    pub(crate) type_id_to_uuid: HashMap<TypeId, Uuid>,
    pub clone_fns: HashMap<TypeId, CloneFn>,
    #[cfg(feature = "serde")]
    pub(crate) serialize_fns: HashMap<Uuid, crate::serialize::SerializeFns>,
}
impl RegistryInner {
    fn new() -> Self {
        Self {
            type_id_to_uuid: Default::default(),
            clone_fns: Default::default(),
            serialize_fns: Default::default(),
        }
    }
}
#[derive(Clone)]
pub struct Registry {
    pub(crate) inner: Arc<RegistryInner>,
}
impl Registry {
    pub fn builder() -> RegistryBuilder {
        RegistryBuilder::default()
    }
    pub fn type_id_to_uuid(&self, type_id: TypeId) -> Option<&Uuid> {
        self.inner.type_id_to_uuid.get(&type_id)
    }
    pub fn clone_entity(&self, entity_ref: EntityRef) -> EntityBuilder {
        let mut builder = EntityBuilder::new();

        for component_type in entity_ref.component_types() {
            if let Some(clone_fn) = self.inner.clone_fns.get(&component_type) {
                clone_fn.into_builder(entity_ref, &mut builder);
            }
        }

        builder
    }
    pub fn clone_component_untyped(
        &self,
        type_id: TypeId,
        src: EntityRef<'_>,
        dst: Entity,
        dst_world: &mut World,
    ) -> Result<(), NoSuchEntity> {
        let clone_fn = self.inner.clone_fns.get(&type_id).unwrap();
        clone_fn.into_world(src, dst, dst_world)
    }
    pub fn clone_component<C: CloneComponent>(
        &self,
        src: EntityRef<'_>,
        dst: Entity,
        dst_world: &mut World,
    ) -> Result<(), NoSuchEntity> {
        self.clone_component_untyped(TypeId::of::<C>(), src, dst, dst_world)
    }
}

pub struct RegistryBuilder {
    pub(crate) registry: RegistryInner,
}

impl Default for RegistryBuilder {
    fn default() -> Self {
        let mut builder = Self {
            registry: RegistryInner::new(),
        };
        builder.register_clone::<Uuid>();
        builder
    }
}

impl RegistryBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register_clone<C: CloneComponent>(&mut self) {
        self.registry.clone_fns.insert(
            TypeId::of::<C>(),
            CloneFn {
                builder: |entity, out| {
                    if let Some(component) = entity.get::<&C>() {
                        let cloned = (*component).clone();
                        out.add(cloned);
                    }
                },
                world: |entity, dst, dst_world| -> Result<(), NoSuchEntity> {
                    if let Some(component) = entity.get::<&C>() {
                        let cloned = (*component).clone();
                        dst_world.add_component(dst, cloned)?;
                    }
                    Ok(())
                },
            },
        );
    }
    pub fn build(self) -> Registry {
        Registry {
            inner: Arc::new(self.registry),
        }
    }
}
