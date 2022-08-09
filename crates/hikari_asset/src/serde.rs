use std::marker::PhantomData;

use once_cell::sync::OnceCell;
use serde::{
    de::{self, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};

use crate::{AssetManager, Handle, Asset};

static ASSET_MANAGER: OnceCell<AssetManager> = OnceCell::new();

pub fn init(ass_man: AssetManager) {
    ASSET_MANAGER.get_or_init(|| ass_man);
}
impl<T: Asset> Serialize for Handle<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Handle", 2)?;
        let ass_man = ASSET_MANAGER.get().expect("Asset Manager not initialized");

        state.serialize_field("uuid", &ass_man.get_uuid(self))?;
        state.serialize_field("path", &ass_man.get_path(self))?;
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

        let ass_man = ASSET_MANAGER.get().expect("Asset Manager has not been set");

        let handle = ass_man
            .load(&path)
            .map_err(|err| de::Error::custom(&format!("Failed to load asset: {}", err)))?;

        let loader_uuid = ass_man.get_uuid(&handle);
        
        //assert!(uuid == loader_uuid, "{}", path.display());

        Ok(handle)
    }
}
