use serde::{
    de::{DeserializeSeed, Visitor},
    ser::SerializeSeq,
    Deserializer, Serialize,
};

use crate::AssetDB;

use super::{AnySerde, RecordDeserializer};

impl AssetDB {
    pub fn as_serializable<'db, 'a>(
        &'db self,
        any_serde: &'a AnySerde,
    ) -> SerializableAssetDB<'db, 'a> {
        SerializableAssetDB {
            db: self,
            any_serde,
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
        any_serde: &AnySerde,
    ) -> Result<Self, D::Error> {
        AssetDBDeserializer { any_serde }.deserialize(deserializer)
    }
}

pub struct SerializableAssetDB<'db, 'a> {
    db: &'db AssetDB,
    any_serde: &'a AnySerde,
}
impl<'db, 'a> Serialize for SerializableAssetDB<'db, 'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let any_serde = &self.any_serde;
        let mut records_list = serializer.serialize_seq(Some(self.db.records.len()))?;

        for record in &self.db.records {
            records_list.serialize_element(&record.as_serializable(any_serde))?;
        }

        records_list.end()
    }
}
pub struct AssetDBDeserializer<'a> {
    any_serde: &'a AnySerde,
}
impl<'a, 'de> DeserializeSeed<'de> for AssetDBDeserializer<'a> {
    type Value = AssetDB;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(AssetDBVisitor(self.any_serde))
    }
}
struct AssetDBVisitor<'a>(&'a AnySerde);

impl<'a, 'de> Visitor<'de> for AssetDBVisitor<'a> {
    type Value = AssetDB;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("An AssetDB")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let any_serde = self.0;

        let mut db = AssetDB::new();

        while let Some(record) =
            seq.next_element_seed::<RecordDeserializer>(RecordDeserializer { any_serde })?
        {
            db.records.push(record);
        }

        db.build_query_accelerators();

        Ok(db)
    }
}
