#![allow(unused)]
use std::{any::TypeId, collections::{HashMap, hash_map::Iter}};

use hikari::core::{Component, ComponentError, Entity, NoSuchEntity, World};
use hikari_imgui::Ui;

use crate::EngineState;

use super::Editor;
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
    pub fn iter(&self) -> Iter<'_, TypeId, ComponentDispatch> {
        self.dispatchers.iter()
    }
}
pub struct ComponentDispatch {
    name: fn() -> &'static str,
    add_component: fn(Entity, &mut World) -> Result<(), NoSuchEntity>,
    draw_component: fn(&Ui, Entity, &mut World, &mut Editor, EngineState) -> anyhow::Result<()>,
    remove_component: fn(Entity, &mut World) -> Result<(), ComponentError>,
    clone_component: fn(Entity, &mut World, &mut World) -> Result<(), ComponentError>,
}
impl ComponentDispatch {
    pub fn new<T: EditorComponent>() -> Self {
        Self {
            name: T::name,
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
        src: &mut World,
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
    fn draw(&mut self, ui: &Ui, entity: Entity, editor: &mut Editor, state: EngineState) -> anyhow::Result<()>;
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
    fn clone_component(entity: Entity, src: &mut World, dst: &mut World) -> Result<(), ComponentError>
    where
        Self: Sized,
    {
        let cloned_component = src.get_component::<Self>(entity)?.clone();
        dst.create_entity_at(entity, (cloned_component,));
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
        let mut component = world.get_component_mut::<Self>(entity).unwrap();
        component.draw(ui, entity, editor, state)
    }
}
