use crate::DirLight;
use hikari_math::{Mat4, Vec2, Vec3A, Vec3, Vec4};

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
    pub viewport_size: Vec2,
    pub exposure: f32,
    pub environment_intensity: f32,
    pub env_map_ix: u32,
    pub env_map_irradiance_ix: u32,
    pub env_map_prefiltered_ix: u32,
    pub brdf_lut_ix: u32,
    pub dir_light: DirLight,
    pub show_cascades: u32,
}
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct MaterialInputs {
    pub albedo: Vec4,
    pub emissive: Vec3,
    pub roughness: f32,
    pub metallic: f32,
    pub uv_set: u32,
    pub albedo_ix: i32,
    pub emissive_ix: i32,
    pub roughness_ix: i32,
    pub metallic_ix: i32,
    pub normal_ix: i32,
}
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct PushConstants {
    pub transform: Mat4,
    pub mat: MaterialInputs
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PerInstanceData {
    pub transform: Mat4,
}