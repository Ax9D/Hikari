use serde::{
    de::{DeserializeSeed, Visitor},
    ser::{SerializeSeq},
    Deserializer, Serialize,
};

use crate::{AssetDB, Record};

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
        let any_serde = self.any_serde;
        RecordsList {
            records: &self.db.records,
            any_serde
        }.serialize(serializer)
    }
}

struct RecordsList<'records, 'a> {
    records: &'records [Record],
    any_serde: &'a AnySerde
}
impl<'db, 'a> Serialize for RecordsList<'db, 'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut records_list = serializer.serialize_seq(Some(self.records.len()))?;
        for record in self.records {
            records_list.serialize_element(&record.as_serializable(self.any_serde))?;
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

        while let Some(record) = seq.next_element_seed::<RecordDeserializer>(RecordDeserializer { any_serde })?
        {
            db.records.push(record);
        }

        // for (ix, record) in records.iter().enumerate() {
        //     let extension = record.path.extension().unwrap();
        //     let extension = extension.to_str().unwrap();

        //     let uuid = match extension {
        //         "hmat" => "4619d2c4-246f-4dc6-acb1-6d34633f7b71",
        //         "png" | "jpg" | "jpeg" | "dds" | "bmp" | "gif" | "tga" => "70ff2d10-3fc1-4851-9fb7-521d8cd49ad5",
        //         "hdr" => "1928ab7d-2dfc-438e-ae23-bf1af8e9866e",
        //         "gltf" | "glb" => "90eff7a8-4a6b-444f-bc09-dbc441bda057",
        //         "hscene" => "c2103fb7-13fe-4f18-84a5-f6075ce7206a",
        //         "cubemap" => "80ced31a-1eaf-424c-9e78-0bcd3742ba43",
        //         un=> unreachable!("{}", un)
        //     };

        //     db.type_uuid_to_records.entry(Uuid::from_str(uuid).unwrap())
        //     .or_default()
        //     .push(ix);
        // }
    
        db.build_query_accelerators();

        Ok(db)
    }
}
struct RecordsListDeserializer<'a> {
    any_serde: &'a AnySerde,
}
impl<'a, 'de> DeserializeSeed<'de> for RecordsListDeserializer<'a> {
    type Value = Vec<Record>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de> {
        deserializer.deserialize_seq(self)
    }
}
impl<'a, 'de> Visitor<'de> for RecordsListDeserializer<'a> {
    type Value = Vec<Record>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A Record List")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>, {
        let mut records = Vec::new();

        while let Some(record) =
            seq.next_element_seed::<RecordDeserializer>(RecordDeserializer { any_serde: &self.any_serde })?
        {
            records.push(record);
        }

        Ok(records)
    }
}