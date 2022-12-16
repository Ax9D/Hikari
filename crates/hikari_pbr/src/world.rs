use crate::DirLight;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct WorldUBO {
    pub camera_position: hikari_math::Vec3A,
    pub view: [f32; 16],
    pub view_proj: [f32; 16],
    pub camera_near: f32,
    pub camera_far: f32,
    pub exposure: f32,
    pub _pad: f32,
    pub dir_light: DirLight,
    pub show_cascades: u32,
}