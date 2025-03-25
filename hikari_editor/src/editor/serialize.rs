use super::{windows::{Outliner, EditorSettings, RenderSettings}, Editor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(serde::Serialize)]
#[serde(default)]
struct SerializedEditor<'a> {
    outliner: &'a Outliner,
    editor_settings: &'a EditorSettings,
    render_settings: &'a RenderSettings,
    //viewport: &'a Viewport
}
#[derive(Default, serde::Deserialize)]
#[serde(default)]
struct DeserializedEditor {
    outliner: Outliner,
    editor_settings: EditorSettings,
    render_settings: RenderSettings,
    //viewport: Viewport
}

impl Editor {
    fn into_serializable(&self) -> SerializedEditor {
        SerializedEditor {
            outliner: &self.outliner,
            editor_settings: &self.editor_settings,
            render_settings: &self.render_settings,
            //viewport: &self.viewport,
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
        self.editor_settings = deserialized.editor_settings;
        self.render_settings = deserialized.render_settings;
        //self.viewport = deserialized.viewport;

        Ok(())
    }
}
