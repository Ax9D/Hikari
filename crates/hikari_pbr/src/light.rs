use crate::passes::shadow::N_CASCADES;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct DirLight {
    pub intensity: f32,
    pub size: f32,
    pub normal_bias: f32,
    pub cast_shadows: u32,
    pub max_shadow_distance: f32,
    pub shadow_fade: f32,
    pub cascade_split_lambda: f32,
    _pad: f32,
    pub color: hikari_math::Vec3A,
    pub direction: hikari_math::Vec3A,
    pub up_direction: hikari_math::Vec3A,
    pub cascades: [ShadowCascade; N_CASCADES],
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct ShadowCascade {
    pub map_size: f32,
    pub map_texel_size: f32,
    pub atlas_uv_offset: hikari_math::Vec2,
    pub atlas_size_ratio: hikari_math::Vec2,
    _pad: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct CascadeRenderInfo {
    split: f32,
    near: f32,
    far: f32,
    _pad: f32,
    view: [f32; 16],
    view_proj: [f32; 16],
}
