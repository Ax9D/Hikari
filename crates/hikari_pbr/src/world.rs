use crate::DirLight;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct WorldUBO {
    pub camera_position: hikari_math::Vec3A,
    pub view: [f32; 16],
    pub view_proj: [f32; 16],
    pub exposure: f32,
    pub _pad: hikari_math::Vec3,
    pub dir_light: DirLight,
    pub show_cascades: u32,
}