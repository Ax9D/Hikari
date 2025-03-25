use serde::{Deserialize, Serialize, Serializer};
use std::{any::Any, collections::HashMap};
use type_uuid::TypeUuid;

struct SerdeFns {
    serialize: fn(&(dyn Any + Send + Sync), &mut dyn FnMut(&dyn erased_serde::Serialize)),
    deserialize: fn(
        &mut dyn erased_serde::Deserializer,
    ) -> Result<Box<dyn Any + Send + Sync>, erased_serde::Error>,
}
impl SerdeFns {
    pub fn serialize<S: Serializer>(
        &self,
        untyped_data: &(dyn Any + Send + Sync),
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut serializer = Some(serializer);
        let mut result = None;
        (self.serialize)(untyped_data, &mut |dyn_data| {
            result = Some(erased_serde::serialize(
                dyn_data,
                serializer.take().unwrap(),
            ));
        });
        result.unwrap()
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        &self,
        deserializer: D,
    ) -> Result<Box<dyn Any + Send + Sync>, D::Error> {
        let mut erased_deserializer = <dyn erased_serde::Deserializer>::erase(deserializer);

        let result = (self.deserialize)(&mut erased_deserializer);

        result.map_err(serde::de::Error::custom)
    }
}
#[derive(Default)]
pub struct AnySerde {
    fn_map: HashMap<type_uuid::Bytes, SerdeFns, fxhash::FxBuildHasher>,
}

impl AnySerde {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register_type<
        T: Serialize + for<'a> Deserialize<'a> + TypeUuid + Send + Sync + 'static,
    >(
        &mut self,
    ) {
        let fns = SerdeFns {
            serialize: |data, serializer_fn| {
                let typed_data = data.downcast_ref::<T>().unwrap();
                (serializer_fn)(typed_data);
            },
            deserialize: |deserializer| -> Result<Box<dyn Any + Send + Sync>, erased_serde::Error> {
                let typed_data = erased_serde::deserialize::<T>(deserializer)?;
                Ok(Box::new(typed_data))
            },
        };
        self.fn_map.insert(T::UUID, fns);
    }
    pub fn serialize_any<S: Serializer>(
        &self,
        type_id: &type_uuid::Bytes,
        untyped_data: &(dyn Any + Send + Sync),
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let serde_fns = self.fn_map.get(type_id).expect("Unregistered Type");
        serde_fns.serialize(untyped_data, serializer)
    }
    pub fn deserialize_any<'de, D: serde::Deserializer<'de>>(
        &self,
        type_id: &type_uuid::Bytes,
        deserializer: D,
    ) -> Result<Box<dyn Any + Send + Sync>, D::Error> {
        let serde_fns = self.fn_map.get(type_id).expect("Unregistered Type");
        serde_fns.deserialize(deserializer)
    }
}
