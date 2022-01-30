#[derive(Clone, Debug)]
pub struct Transform {
    pub position: glam::Vec3,
    pub scale: glam::Vec3,
    pub rotation: glam::Quat,
}
impl Transform {
    pub fn new(position: glam::Vec3, scale: glam::Vec3, rotation: glam::Quat) -> Self {
        Self {
            position: position,
            scale: scale,
            rotation,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::IDENTITY,
        }
    }
}
