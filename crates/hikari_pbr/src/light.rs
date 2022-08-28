use crate::passes::shadow::{MAX_SHADOW_CASCADES};

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct DirLight {
    pub intensity: f32,
    pub size: f32,
    pub constant_bias_factor: f32,
    pub normal_bias_factor: f32,
    pub max_shadow_distance: f32, 
    pub shadow_fade: f32,
    pub color: hikari_math::Vec3A,
    pub direction: hikari_math::Vec3A,
    pub cascades: [ShadowCascade; MAX_SHADOW_CASCADES]
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct ShadowCascade {
    pub split_depth: f32,
    pub near: f32,
    pub far: f32,
    _pad: f32,
    pub view: [f32; 16],
    pub view_proj: [f32; 16]
}