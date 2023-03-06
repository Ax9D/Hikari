#![allow(unused)]
use std::{
    any::TypeId,
    collections::{HashMap},
};

use hikari::core::{Component, ComponentError, Entity, NoSuchEntity, World};
use hikari::imgui::Ui;

use crate::Editor;
use hikari_editor::*;

#[derive(Default)]
pub struct EditorComponents {
    dispatchers: HashMap<TypeId, ComponentDispatch>,
}
impl EditorComponents {
    pub fn register<T: EditorComponent>(&mut self) {
        self.dispatchers
            .insert(TypeId::of::<T>(), ComponentDispatch::new::<T>());
    }
    pub fn get(&self, type_id: TypeId) -> Option<&ComponentDispatch> {
        self.dispatchers.get(&type_id)
    }
    pub fn iter<'a>(&'a self) -> std::collections::hash_map::Iter<'a, TypeId, ComponentDispatch> {
        self.dispatchers.iter()
    }
}

pub struct ComponentDispatch {
    name: fn() -> &'static str,
    type_id: TypeId,
    sort_key: fn() -> usize,
    add_component: fn(Entity, &mut World) -> Result<(), NoSuchEntity>,
    draw_component: fn(&Ui, Entity, &mut World, &mut Editor, EngineState) -> anyhow::Result<()>,
    remove_component: fn(Entity, &mut World) -> Result<(), ComponentError>,
    clone_component: fn(Entity, &World, &mut World) -> Result<(), ComponentError>,
}
impl ComponentDispatch {
    pub fn new<T: EditorComponent>() -> Self {
        Self {
            name: T::name,
            type_id: TypeId::of::<T>(),
            sort_key: T::sort_key,
            add_component: T::add_component,
            draw_component: T::draw_component,
            remove_component: T::remove_component,
            clone_component: T::clone_component,
        }
    }
    #[inline]
    pub fn name(&self) -> &'static str {
        (self.name)()
    }
    #[inline]
    pub fn sort_key(&self) -> usize {
        (self.sort_key)()
    }
    #[inline]
    pub fn add_component(&self, entity: Entity, world: &mut World) -> Result<(), NoSuchEntity> {
        (self.add_component)(entity, world)
    }
    #[inline]
    pub fn remove_component(&self, entity: Entity, world: &mut World) -> Result<(), ComponentError>
    where
        Self: Sized,
    {
        (self.remove_component)(entity, world)
    }
    #[inline]
    pub fn clone_component(
        &self,
        entity: Entity,
        src: &World,
        dst: &mut World,
    ) -> Result<(), ComponentError>
    where
        Self: Sized,
    {
        (self.clone_component)(entity, src, dst)
    }
    #[inline]
    pub fn draw_component(
        &self,
        ui: &Ui,
        entity: Entity,
        world: &mut World,
        editor: &mut Editor,
        state: EngineState,
    ) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        (self.draw_component)(ui, entity, world, editor, state)
    }
}

pub trait EditorComponent: Component {
    fn name() -> &'static str
    where
        Self: Sized;
    fn new() -> Self
    where
        Self: Sized;
    fn sort_key() -> usize { 
        usize::MAX
    }
    fn draw(
        &mut self,
        ui: &Ui,
        entity: Entity,
        editor: &mut Editor,
        state: EngineState,
    ) -> anyhow::Result<()>;
    fn clone(&self) -> Self
    where
        Self: Sized;

    fn add_component(entity: Entity, world: &mut World) -> Result<(), NoSuchEntity>
    where
        Self: Sized,
    {
        world.add_component(entity, Self::new())
    }

    fn remove_component(entity: Entity, world: &mut World) -> Result<(), ComponentError>
    where
        Self: Sized,
    {
        world.remove_component::<Self>(entity)?;

        Ok(())
    }
    fn clone_component(entity: Entity, src: &World, dst: &mut World) -> Result<(), ComponentError>
    where
        Self: Sized,
    {
        let cloned_component = src.get_component::<&Self>(entity)?.clone();
        if dst.entity(entity).is_ok() {
            dst.add_component(entity, cloned_component)?;
        } else {
            dst.create_entity_at(entity, (cloned_component,));
        }
        Ok(())
    }
    fn draw_component(
        ui: &Ui,
        entity: Entity,
        world: &mut World,
        editor: &mut Editor,
        state: EngineState,
    ) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        let mut component = world.get_component::<&mut Self>(entity).unwrap();
        component.draw(ui, entity, editor, state)
    }
}