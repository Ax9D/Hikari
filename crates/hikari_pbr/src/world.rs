use crate::DirLight;
use hikari_math::{Mat4, Vec3A};

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct WorldUBO {
    pub camera_position: Vec3A,
    pub proj: Mat4,
    pub view: Mat4,
    pub view_proj: Mat4,
    pub environment_transform: Mat4,
    pub camera_near: f32,
    pub camera_far: f32,
    pub exposure: f32,
    pub environment_intensity: f32,
    pub dir_light: DirLight,
    pub show_cascades: u32,
}
