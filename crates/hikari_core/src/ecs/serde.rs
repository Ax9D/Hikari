use std::{any::TypeId, collections::HashMap};

use hecs::EntityRef;
use serde::{
    de::{DeserializeSeed, MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};
use type_uuid::TypeUuid;
use uuid::Uuid;

use crate::{Component, Entity, World};

pub trait SerializeComponent: Component + Serialize + for<'de> Deserialize<'de> + TypeUuid {}
impl<T: Component + Serialize + for<'de> Deserialize<'de> + TypeUuid> SerializeComponent for T {}
struct ComponentsSerialize<'r, 'e>(&'r Registry, EntityRef<'e>);

impl<'r, 'e> Serialize for ComponentsSerialize<'r, 'e> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.1.component_types().count()))?;

        for type_id in self.1.component_types() {
            if let Some(uuid) = self.0.type_id_to_uuid(type_id) {
                map.serialize_entry(uuid, &ComponentSerialize(self.0, self.1, *uuid))?;
            } else {
                log::warn!("Skipping serializing typeid: {:#?}", type_id);
            }
        }

        map.end()
    }
}

struct ComponentSerialize<'r, 'e>(&'r Registry, EntityRef<'e>, Uuid);

impl<'r, 'e> Serialize for ComponentSerialize<'r, 'e> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize_component(&self.2, self.1, serializer)
    }
}

#[derive(Clone)]
struct SerializeFns {
    serialize_fn: fn(EntityRef<'_>, &mut dyn FnMut(&dyn erased_serde::Serialize)),
    deserialize_fn: fn(
        Entity,
        &mut World,
        &mut dyn erased_serde::Deserializer,
    ) -> Result<(), erased_serde::Error>,
}
#[derive(Default, Clone)]
pub struct Registry {
    type_id_to_uuid: HashMap<TypeId, Uuid>,
    serialize_fns: HashMap<Uuid, SerializeFns>,
}

impl Registry {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn register_component<C: SerializeComponent>(&mut self) {
        let serialize_fns = SerializeFns {
            serialize_fn: |entity_ref, serialize_fn| {
                let component = entity_ref.get::<C>().unwrap();
                (serialize_fn)(&*component)
            },
            deserialize_fn: |entity, world, deserializer| -> Result<(), erased_serde::Error> {
                let component = erased_serde::deserialize::<C>(deserializer)?;
                world.add_component(entity, component).unwrap();

                Ok(())
            },
        };

        let uuid = Uuid::from_bytes(C::UUID);
        self.type_id_to_uuid.insert(TypeId::of::<C>(), uuid);
        self.serialize_fns.insert(uuid, serialize_fns);
    }
    fn type_id_to_uuid(&self, type_id: TypeId) -> Option<&Uuid> {
        self.type_id_to_uuid.get(&type_id)
    }
    pub fn serialize_component<S: Serializer>(
        &self,
        component_id: &Uuid,
        entity_ref: EntityRef,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let serialize_fns = self.serialize_fns.get(component_id).unwrap();
        let mut serializer = Some(serializer);
        let mut result = None;

        let result_cl = &mut result;

        (serialize_fns.serialize_fn)(entity_ref, &mut move |component| {
            *result_cl = Some(erased_serde::serialize(
                component,
                serializer.take().unwrap(),
            ));
        });

        result.unwrap()
    }
    pub fn deserialize_component<'de, D: Deserializer<'de>>(
        &self,
        component_id: &Uuid,
        entity: Entity,
        world: &mut World,
        deserializer: D,
    ) -> Result<(), D::Error> {
        if let Some(serialize_fns) = self.serialize_fns.get(component_id) {
            let mut deserializer = <dyn erased_serde::Deserializer>::erase(deserializer);
            (serialize_fns.deserialize_fn)(entity, world, &mut deserializer)
                .map_err(serde::de::Error::custom)?;
        } else {
            log::warn!("Skipping deserializing uuid: {}", component_id);
        }
        Ok(())
    }
}

pub struct SerializableWorld<'w, 'r> {
    world: &'w World,
    registry: &'r Registry,
}

impl<'w, 'r> SerializableWorld<'w, 'r> {
    pub(crate) fn new(world: &'w World, registry: &'r Registry) -> Self {
        Self { world, registry }
    }
}

impl World {
    pub fn as_serializable<'w, 'r>(&'w self, registry: &'r Registry) -> SerializableWorld<'w, 'r> {
        SerializableWorld::new(self, registry)
    }
}

impl<'w, 'r> Serialize for SerializableWorld<'w, 'r> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.world.len()))?;
        for entity_ref in self.world.entities() {
            map.serialize_entry(
                &entity_ref.entity(),
                &ComponentsSerialize(&self.registry, entity_ref),
            )?;
        }

        map.end()
    }
}

impl World {
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
        registry: &Registry,
    ) -> Result<Self, D::Error> {
        let mut world = Self::new();
        WorldDeserializer {
            world: &mut world,
            registry,
        }
        .deserialize(deserializer)?;

        Ok(world)
    }
}
pub struct WorldDeserializer<'w, 'r> {
    pub world: &'w mut World,
    pub registry: &'r Registry,
}
impl<'w, 'r, 'de> DeserializeSeed<'de> for WorldDeserializer<'w, 'r> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(WorldVisitor {
            world: self.world,
            registry: self.registry,
        })?;
        Ok(())
    }
}
struct WorldVisitor<'w, 'r> {
    world: &'w mut World,
    registry: &'r Registry,
}
impl<'w, 'r, 'de> Visitor<'de> for WorldVisitor<'w, 'r> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A world")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some(entity) = map.next_key::<Entity>()? {
            self.world.create_entity_at(entity, ());

            map.next_value_seed(ComponentsDeserializer(entity, self.world, self.registry))?;
        }
        Ok(())
    }
}

struct ComponentsDeserializer<'w, 'r>(Entity, &'w mut World, &'r Registry);

impl<'w, 'r, 'de> DeserializeSeed<'de> for ComponentsDeserializer<'w, 'r> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ComponentsVisitor(self.0, self.1, self.2))?;
        Ok(())
    }
}

struct ComponentsVisitor<'w, 'r>(Entity, &'w mut World, &'r Registry);

impl<'w, 'r, 'de> Visitor<'de> for ComponentsVisitor<'w, 'r> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("An entity's components")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some(uuid) = map.next_key::<uuid::Uuid>()? {
            map.next_value_seed::<ComponentDeserializer>(ComponentDeserializer(
                uuid, self.0, self.1, self.2,
            ))?;
        }

        Ok(())
    }
}
struct ComponentDeserializer<'w, 'r>(Uuid, Entity, &'w mut World, &'r Registry);

impl<'w, 'r, 'de> DeserializeSeed<'de> for ComponentDeserializer<'w, 'r> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.3
            .deserialize_component(&self.0, self.1, self.2, deserializer)?;

        Ok(())
    }
}

#[test]
fn serialize_component() {
    #[derive(Serialize, Deserialize, TypeUuid)]
    #[uuid = "af785417-dd52-4397-ba55-f747c6d67fc9"]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Serialize, Deserialize, TypeUuid)]
    #[uuid = "3042c403-106d-4c2d-a291-d879a7ae014f"]
    struct Id {
        name: String,
    }

    let mut world = World::new();
    let entity = world.create_entity();

    world
        .add_component(entity, Position { x: 0.5, y: 0.0 })
        .unwrap();
    world
        .add_component(entity, Id { name: "Foo".into() })
        .unwrap();

    let mut registry = Registry::new();
    registry.register_component::<Position>();
    registry.register_component::<Id>();

    fn component_to_string(
        component: &Uuid,
        entity: Entity,
        world: &World,
        registry: &Registry,
    ) -> String {
        let string_buffer = std::io::BufWriter::new(Vec::new());
        let mut yaml_ser = serde_yaml::Serializer::new(string_buffer);

        registry
            .serialize_component(component, world.entity(entity).unwrap(), &mut yaml_ser)
            .expect("Failed to serialize");

        let string_buffer = yaml_ser.into_inner();

        String::from_utf8(string_buffer.into_inner().unwrap()).unwrap()
    }

    assert_eq!(
        serde_yaml::to_string(&Position { x: 0.5, y: 0.0 }).unwrap(),
        component_to_string(&Uuid::from_bytes(Position::UUID), entity, &world, &registry)
    );
    assert_eq!(
        serde_yaml::to_string(&Id { name: "Foo".into() }).unwrap(),
        component_to_string(&Uuid::from_bytes(Id::UUID), entity, &world, &registry)
    );
}
#[test]
fn round_trip() {
    #[derive(Serialize, Deserialize, TypeUuid, PartialEq, Debug)]
    #[uuid = "af785417-dd52-4397-ba55-f747c6d67fc9"]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Serialize, Deserialize, TypeUuid, PartialEq, Eq, Debug)]
    #[uuid = "3042c403-106d-4c2d-a291-d879a7ae014f"]
    struct Id {
        name: String,
    }

    let mut world_in = World::new();
    let entity = world_in.create_entity();

    world_in
        .add_component(entity, Position { x: 0.5, y: 0.0 })
        .unwrap();
    world_in
        .add_component(entity, Id { name: "Foo".into() })
        .unwrap();

    let mut registry = Registry::new();
    registry.register_component::<Position>();
    registry.register_component::<Id>();

    let world_string = serde_yaml::to_string(&world_in.as_serializable(&registry)).unwrap();

    let deserializer = serde_yaml::Deserializer::from_str(&world_string);
    let world_out = World::deserialize(deserializer, &registry).unwrap();

    assert!(world_out.contains(entity));
    let position = world_out.get_component::<Position>(entity);
    assert!(position.is_ok());
    let position = position.unwrap();

    assert_eq!(&Position { x: 0.5, y: 0.0 }, &*position);

    let id = world_out.get_component::<Id>(entity);
    assert!(id.is_ok());
    let id = id.unwrap();

    assert_eq!(&Id { name: "Foo".into() }, &*id);
}
