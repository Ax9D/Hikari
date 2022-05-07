struct SceneSerializer {}
pub use hecs::serialize::row::*;
use serde::{ser::SerializeMap, Serialize};

#[derive(Serialize)]
pub enum Components {
    Transform,
}

impl SerializeContext for SceneSerializer {
    fn serialize_entity<S>(
        &mut self,
        entity: hecs::EntityRef<'_>,
        map: &mut S,
    ) -> Result<(), S::Error>
    where
        S: SerializeMap,
    {
        try_serialize::<f32, _, _>(&entity, &Components::Transform, map)
    }
}
