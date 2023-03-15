use super::{windows::Outliner, Editor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(serde::Serialize)]
#[serde(default)]
struct SerializedEditor<'a> {
    outliner: &'a Outliner,
}
#[derive(Default, serde::Deserialize)]
#[serde(default)]
struct DeserializedEditor {
    outliner: Outliner,
}

impl Editor {
    fn into_serializable(&self) -> SerializedEditor {
        SerializedEditor {
            outliner: &self.outliner,
        }
    }
    pub fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let serializable = self.into_serializable();
        serializable.serialize(serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        &mut self,
        deserializer: D,
    ) -> Result<(), D::Error> {
        let deserialized = DeserializedEditor::deserialize(deserializer)?;

        self.outliner = deserialized.outliner;

        Ok(())
    }
}
