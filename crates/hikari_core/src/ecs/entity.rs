use uuid::Uuid;

#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default)
)]
pub struct EntityId {
    pub name: String,
    pub uuid: Uuid
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new_with_name("untitled")
    }
}

impl EntityId {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_with_name(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            uuid: Uuid::new_v4()
        }
    }
}