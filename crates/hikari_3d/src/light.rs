use hikari_math::Vec4;

#[derive(Clone, Copy, Debug, type_uuid::TypeUuid)]
#[uuid = "205dd658-14f4-49eb-9e3a-d6cd0fd128ab"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Light {
    pub color: Vec4,
    pub intensity: f32,
    pub cast_shadows: bool,
    pub kind: LightKind,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            color: Vec4::ONE,
            intensity: 1.0,
            cast_shadows: true,
            kind: LightKind::Directional,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LightKind {
    Point,
    Directional,
}
