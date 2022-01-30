#[derive(Clone, Copy, Debug)]
pub struct Light {
    pub color: glam::Vec4,
    pub intensity: f32,
    pub cast_shadows: bool,
    pub kind: LightKind
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightKind {
    Point,
    Directional,
}