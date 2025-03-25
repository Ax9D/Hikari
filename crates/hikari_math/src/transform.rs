use crate::*;
#[derive(Copy, Clone, Debug, type_uuid::TypeUuid)]
#[uuid = "d8c0dc46-38ad-430b-8eeb-790bf5ad44d3"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
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
    pub fn from_matrix(mat: Mat4) -> Self {
        let (scale, rotation, position) = mat.to_scale_rotation_translation();
        Self {
            position,
            scale,
            rotation,
        }
    }
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }
    #[inline]
    pub fn get_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
    #[inline]
    pub fn get_rotation_matrix(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, Vec3::ZERO)
    }
}
