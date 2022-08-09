use std::{sync::Arc};

use hikari::{
    asset::{Asset, Loader, Saver},
    core::{
        serde::{Registry, WorldDeserializer},
        World,
    },
};
use serde::{de::Visitor, ser::SerializeMap, Deserializer, Serialize, Serializer};

pub const SCENE_EXTENSION: &str = "hscene";
pub struct Scene {
    pub world: World,
}
impl Scene {
    // #[allow(unused)]
    // pub fn save(&self, path: impl AsRef<Path>, registry: &Registry) -> anyhow::Result<()> {
    //     serde_yaml::to_writer(std::fs::File::open(path)?, &SerializableScene {
    //         scene: &self,
    //         registry
    //     })?;

    //     Ok(())
    // }
    fn serialize<S: Serializer>(&self, ser: S, registry: &Registry) -> Result<S::Ok, S::Error> {
        SerializableScene {
            scene: &self,
            registry,
        }
        .serialize(ser)
    }
    fn deserialize<'de, D: Deserializer<'de>>(
        de: D,
        registry: &Registry,
    ) -> Result<Scene, D::Error> {
        de.deserialize_map(SceneVisitor { registry })
    }
}
struct SerializableScene<'s, 'r> {
    scene: &'s Scene,
    registry: &'r Registry,
}
impl<'s, 'r> Serialize for SerializableScene<'s, 'r> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("world", &self.scene.world.as_serializable(self.registry))?;

        map.end()
    }
}
struct SceneVisitor<'r> {
    registry: &'r Registry,
}
impl<'r, 'de> Visitor<'de> for SceneVisitor<'r> {
    type Value = Scene;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A Scene")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut world_fd = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "world" => {
                    if world_fd.is_some() {
                        return Err(serde::de::Error::duplicate_field("world"));
                    }
                    let mut world = World::new();
                    map.next_value_seed(WorldDeserializer {
                        registry: self.registry,
                        world: &mut world,
                    })?;

                    world_fd = Some(world);
                }
                field => {
                    return Err(serde::de::Error::unknown_field(field, &["world"]));
                }
            }
        }
        let world = world_fd.ok_or_else(|| serde::de::Error::missing_field("world"))?;

        Ok(Scene { world })
    }
}

impl Asset for Scene {
    type Settings = ();
}
#[derive(Clone)]
pub struct SceneLoader {
    pub registry: Arc<Registry>,
}

impl Loader for SceneLoader {
    fn extensions(&self) -> &[&str] {
        &[SCENE_EXTENSION]
    }

    fn load(&self, ctx: &mut hikari::asset::LoadContext) -> anyhow::Result<()> {
        let de = serde_yaml::Deserializer::from_reader(ctx.reader());

        let scene = Scene::deserialize(de, &self.registry)?;

        ctx.set_asset(scene);

        Ok(())
    }
}
impl Saver for SceneLoader {
    fn extensions(&self) -> &[&str] {
        <Self as Loader>::extensions(&self)
    }

    fn save(
        &self,
        context: &mut hikari::asset::SaveContext,
        writer: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let scene = context.get_asset::<Scene>();
        let mut ser = serde_yaml::Serializer::new(writer);
        scene.serialize(&mut ser, &self.registry)?;

        let mut ser = serde_yaml::Serializer::new(Vec::new());
        scene.serialize(&mut ser, &self.registry)?;
        println!("{}", String::from_utf8(ser.into_inner()).unwrap());

        Ok(())
    }
}
