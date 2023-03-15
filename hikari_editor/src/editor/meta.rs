use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "2e8af5b4-6f20-477d-bab2-dcf3df750b8d"]
pub struct EditorOnly;

#[derive(Default, Clone, Serialize, Deserialize, TypeUuid)]
#[serde(default)]
#[uuid = "b1afd17a-752d-4df5-a626-ee43e8041c9c"]
pub struct EditorInfo {
    pub name: String,
    pub index: usize,
}

impl EditorInfo {
    pub fn new(name: impl AsRef<str>, index: usize) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            index
        }
    }
}
// #[derive(Default, Clone, Serialize, Deserialize, TypeUuid)]
// #[serde(default)]
// #[uuid = "9215dd05-f049-4e8f-8899-6ad92aeead47"]
// pub struct EditorEntityInfo {
//     pub order: Vec<Entity>
// }