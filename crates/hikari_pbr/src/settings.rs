#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default)
)]
pub struct DebugSettings {
    pub show_shadow_cascades: bool,
    pub view: DebugView,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
)]
pub enum ShadowResolution {
    D256 = 0,
    D512,
    D1024,
    #[default]
    D2048,
    D4096,
}

impl ShadowResolution {
    pub const fn size(self) -> u32 {
        match self {
            ShadowResolution::D256 => 256,
            ShadowResolution::D512 => 512,
            ShadowResolution::D1024 => 1024,
            ShadowResolution::D2048 => 2048,
            ShadowResolution::D4096 => 4096,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
)]
pub enum DebugView {
    #[default]
    None = 0,
    Unlit,
    Wireframe,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default)
)]
pub struct Settings {
    pub fxaa: bool,
    pub vsync: bool,
    pub directional_shadow_map_resolution: ShadowResolution,
    pub debug: DebugSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fxaa: true,
            vsync: true,
            directional_shadow_map_resolution: Default::default(),
            debug: DebugSettings::default()
        }
    }
}
impl Settings {
    pub fn new() -> Self {
        Self::default()
    }
}