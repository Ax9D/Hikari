#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
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
    #[inline]
    pub fn get_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}
