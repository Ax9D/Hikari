use crate::Component;

pub type Entity = hecs::Entity;

pub use hecs::CommandBuffer;

pub struct Scene {
    world: hecs::World,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            world: hecs::World::default(),
        }
    }
    #[inline]
    pub fn create_entity(&mut self) -> Entity {
        self.world.spawn(())
    }
    #[inline]
    pub fn remove_entity(&mut self, entity: Entity) -> Result<(), hecs::NoSuchEntity> {
        self.world.despawn(entity)
    }
    #[inline]
    pub fn add_component(
        &mut self,
        entity: Entity,
        component: impl Component,
    ) -> Result<(), hecs::NoSuchEntity> {
        self.world.insert_one(entity, component)
    }
    #[inline]
    pub fn remove_component<C: Component>(
        &mut self,
        entity: Entity,
    ) -> Result<C, hecs::ComponentError> {
        self.world.remove_one::<C>(entity)
    }
    #[inline]
    pub fn get_component<C: Component>(
        &self,
        entity: Entity,
    ) -> Result<hecs::Ref<'_, C>, hecs::ComponentError> {
        self.world.get(entity)
    }
    #[inline]
    pub fn get_component_mut<C: Component>(
        &self,
        entity: Entity,
    ) -> Result<hecs::RefMut<'_, C>, hecs::ComponentError> {
        self.world.get_mut(entity)
    }
    #[inline]
    pub fn run_query<Q: hecs::Query>(&self, mut f: impl FnMut(Entity, hecs::QueryItem<Q>)) {
        for (entity, item) in self.query::<Q>().iter() {
            (f)(entity, item);
        }
    }
    #[inline]
    pub fn run_query_mut<Q: hecs::Query>(&mut self, mut f: impl FnMut(Entity, hecs::QueryItem<Q>)) {
        for (entity, item) in self.query_mut::<Q>() {
            (f)(entity, item);
        }
    }
    #[inline]
    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.world.query::<Q>()
    }
    #[inline]
    pub fn query_mut<Q: hecs::Query>(&mut self) -> hecs::QueryMut<'_, Q> {
        self.world.query_mut::<Q>()
    }

    #[inline]
    pub fn execute_commands(&mut self, mut cmd: hecs::CommandBuffer) {
        cmd.run_on(&mut self.world);
    }
}
