use std::{any::Any, path::PathBuf};

use serde::{
    de::{DeserializeSeed, Visitor},
    ser::SerializeMap,
    Deserializer, Serialize,
};
use uuid::Uuid;

use crate::record::Record;

use super::helpers::AnySerde;

impl Record {
    pub fn as_serializable<'a, 'record>(
        &'record self,
        any_serde: &'a AnySerde,
    ) -> SerializableRecord<'a, 'record> {
        SerializableRecord {
            any_serde,
            record: self,
        }
    }
    pub fn deserialize<'a, 'id, 'de, D: Deserializer<'de>>(
        deserializer: D,
        any_serde: &'a AnySerde,
    ) -> Result<Self, D::Error> {
        RecordDeserializer { any_serde }.deserialize(deserializer)
    }
}

pub struct SerializableRecord<'a, 'record> {
    any_serde: &'a AnySerde,
    record: &'record Record,
}

impl<'a, 'record> Serialize for SerializableRecord<'a, 'record> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("uuid", &self.record.uuid)?;
        map.serialize_entry("path", &self.record.path)?;

        let settings: &(dyn Any + Send + Sync) = &*self.record.settings;

        map.serialize_entry(
            &Uuid::from_bytes_ref(&self.record.settings_typeid),
            &SerializableSettings(&self.any_serde, &self.record.settings_typeid, settings),
        )?;

        map.end()
    }
}
struct SerializableSettings<'a, 'id, 'settings>(
    &'a AnySerde,
    &'id type_uuid::Bytes,
    &'settings (dyn Any + Send + Sync),
);

impl<'a, 'id, 'settings> Serialize for SerializableSettings<'a, 'id, 'settings> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize_any(self.1, self.2, serializer)
    }
}

pub struct RecordDeserializer<'a> {
    pub any_serde: &'a AnySerde,
}

impl<'a, 'de> DeserializeSeed<'de> for RecordDeserializer<'a> {
    type Value = Record;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(RecordVisitor(self.any_serde))
    }
}

struct RecordVisitor<'a>(&'a AnySerde);

impl<'a, 'de> Visitor<'de> for RecordVisitor<'a> {
    type Value = Record;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A Record")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let Some((_, uuid)) = map.next_entry::<String, Uuid>()? else {
            return Err(serde::de::Error::custom("Expecting uuid"));
        };

        let Some((_, path)) = map.next_entry::<String, PathBuf>()? else {
            return Err(serde::de::Error::custom("Expecting path"));
        };

        let settings_typeid = map.next_key::<Uuid>()?.unwrap();
        let settings_typeid = settings_typeid.into_bytes();
        let settings = map.next_value_seed(SettingsDeserializer(self.0, &settings_typeid))?;

        Ok(Record {
            uuid,
            path,
            settings_typeid,
            settings,
        })
    }
}

struct SettingsDeserializer<'a, 'record>(&'a AnySerde, &'record type_uuid::Bytes);

impl<'a, 'record, 'de> DeserializeSeed<'de> for SettingsDeserializer<'a, 'record> {
    type Value = Box<dyn Any + Send + Sync>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.0.deserialize_any(self.1, deserializer)
    }
}
