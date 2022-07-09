use hikari_math::Vec4;

#[derive(Clone, Copy, Debug)]
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
pub enum LightKind {
    Point,
    Directional,
}
