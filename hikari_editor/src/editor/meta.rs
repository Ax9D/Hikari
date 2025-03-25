use hikari::core::Entity;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "2e8af5b4-6f20-477d-bab2-dcf3df750b8d"]
pub struct EditorOnly;

#[derive(Default, Clone, Serialize, Deserialize, TypeUuid)]
#[serde(default)]
#[uuid = "9215dd05-f049-4e8f-8899-6ad92aeead47"]
pub struct EditorOutlinerInfo {
    pub order: Vec<Entity>
}
