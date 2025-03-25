pub enum Quality {
    Low,
    Medium,
    High,
    Ultra,
}
pub struct GraphicsSettings {
    pub vsync: bool,
    pub fxaa: bool,
    pub shadow_quality: Quality,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            vsync: true,
            fxaa: true,
            shadow_quality: Quality::High
        }
    }
}
impl GraphicsSettings {
    pub fn low() -> Self {
        Self {
            shadow_quality: Quality::Low,
            ..Default::default()
        }
    }
    pub fn medium() -> Self {
        Self {
            shadow_quality: Quality::Medium,
            ..Default::default()
        }
    }
    pub fn high() -> Self {
        Self {
            shadow_quality: Quality::High,
            ..Default::default()
        }
    }
    pub fn ultra() -> Self {
        Self {
            shadow_quality: Quality::Ultra,
            ..Default::default()
        }
    }
    pub fn to_pbr_settings(&self) -> hikari::pbr::Settings {
        use hikari::pbr::ShadowResolution;

        let directional_shadow_map_resolution = match self.shadow_quality {
            Quality::Low => ShadowResolution::D256,
            Quality::Medium => ShadowResolution::D1024,
            Quality::High => ShadowResolution::D2048,
            Quality::Ultra => ShadowResolution::D4096
        };

        hikari::pbr::Settings {
            vsync: self.vsync,
            fxaa: self.fxaa,
            directional_shadow_map_resolution,
            ..Default::default()
        }
    }
}