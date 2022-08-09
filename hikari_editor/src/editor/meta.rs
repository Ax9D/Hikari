use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "b1afd17a-752d-4df5-a626-ee43e8041c9c"]
pub struct EditorInfo {
    pub name: String,
    pub index: usize,
}
#[derive(Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "2e8af5b4-6f20-477d-bab2-dcf3df750b8d"]
pub struct EditorOnly;
