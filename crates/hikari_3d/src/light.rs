use hikari_math::Vec4;

#[derive(Clone, Copy, Debug, type_uuid::TypeUuid)]
#[uuid = "205dd658-14f4-49eb-9e3a-d6cd0fd128ab"]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default)
)]
pub struct Light {
    pub color: Vec4,
    pub intensity: f32,
    pub size: f32,
    pub shadow: ShadowInfo,
    pub kind: LightKind,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            color: Vec4::ONE,
            intensity: 1.0,
            size: 1.0,
            shadow: ShadowInfo::default(),
            kind: LightKind::Directional,
        }
    }
}
#[derive(Clone, Copy, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default)
)]
pub struct ShadowInfo {
    pub enabled: bool,
    pub slope_scaled_bias: f32,
    pub normal_bias: f32,
    pub cascade_split_lambda: f32,
    pub fade: f32,
    pub max_shadow_distance: f32,
    pub cull_front_face: bool,
}
impl Default for ShadowInfo {
    fn default() -> Self {
        Self {
            enabled: true,
            slope_scaled_bias: 5.0,
            normal_bias: 1.0,
            cascade_split_lambda: 0.95,
            fade: 1.0,
            max_shadow_distance: 1000.0,
            cull_front_face: false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LightKind {
    Point,
    Directional,
}
