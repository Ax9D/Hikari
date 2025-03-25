use hikari_math::Vec3;

#[derive(Clone, Copy, Debug, type_uuid::TypeUuid)]
#[uuid = "932bea99-46b4-4b00-b790-f4d5831d8f0d"]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default)
)]
pub struct Outline {
    pub color: Vec3,
    pub thickness: f32
}

impl Default for Outline {
    fn default() -> Self {
        Self {
             color: Vec3::ONE,
             thickness: 2.5
            }
    }
}

impl Outline {
    pub fn new(color: Vec3) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy, Debug, type_uuid::TypeUuid)]
#[uuid = "97c3a525-f22a-4e29-80ad-14ceaa35475f"]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default)
)]
pub struct GroundGrid {

}

impl Default for GroundGrid {
    fn default() -> Self {
        todo!()
    }
}