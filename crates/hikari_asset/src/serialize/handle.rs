use std::marker::PhantomData;

use serde::{
    de::{self, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};

use crate::{Asset, Handle};

impl<T: Asset> Serialize for Handle<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Handle", 2)?;
        let asset_manager = crate::manager::get_asset_manager();
        let asset_db = asset_manager.asset_db().read();
        let erased = self.clone_erased_as_weak();

        state.serialize_field(
            "uuid",
            &asset_db
                .handle_to_uuid(&erased)
                .expect("Unregistered Handle"),
        )?;
        state.serialize_field(
            "path",
            &asset_db
                .handle_to_path(&erased)
                .expect("Unregistered Handle"),
        )?;

        state.end()
    }
}

impl<'de, T: Asset> Deserialize<'de> for Handle<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct("Handle", &["uuid", "path"], HandleVisitor::new())
    }
}

struct HandleVisitor<T> {
    _phantom: PhantomData<T>,
}
impl<T> HandleVisitor<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData::default(),
        }
    }
}

impl<'de, T: Asset> Visitor<'de> for HandleVisitor<T> {
    type Value = Handle<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(&format!("struct Handle<{}>", std::any::type_name::<T>()))
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        use std::path::PathBuf;
        use uuid::Uuid;

        let mut uuid: Option<Uuid> = None;
        let mut path: Option<PathBuf> = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "uuid" => {
                    if uuid.is_some() {
                        return Err(de::Error::duplicate_field("uuid"));
                    }

                    uuid = Some(map.next_value()?);
                }
                "path" => {
                    if path.is_some() {
                        return Err(de::Error::duplicate_field("path"));
                    }

                    path = Some(map.next_value()?);
                }
                _ => return Err(de::Error::unknown_field("uuid", &["uuid", "path"])),
            }
        }

        let uuid = uuid.ok_or_else(|| de::Error::missing_field("uuid"))?;
        let path = path.ok_or_else(|| de::Error::missing_field("path"))?;

        let asset_manager = crate::manager::get_asset_manager();

        let handle = asset_manager
            .load(&path, None, false)
            .map_err(|err| de::Error::custom(&format!("Failed to load asset: {}", err)))?;

        let asset_db = asset_manager.asset_db().read();
        let loader_uuid = asset_db
            .handle_to_uuid(&handle.clone_erased_as_weak())
            .unwrap();

        //assert!(&uuid == loader_uuid, "{}", path.display());

        if &uuid != loader_uuid {
            log::warn!("Inconsistent UUIDs detected for {:?}. This can happen when newly created asset metadata is not saved", path);
        }

        Ok(handle)
    }
}
