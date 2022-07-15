use crate::*;
#[derive(Copy, Clone, Debug, type_uuid::TypeUuid)]
#[uuid = "d8c0dc46-38ad-430b-8eeb-790bf5ad44d3"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }
    #[inline]
    pub fn get_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}
