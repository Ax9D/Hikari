
use hikari_render::{ PerFrame, CpuBuffer};
use crate::{WorldUBO, Settings};

pub struct RenderResources {
    pub settings: Settings,
    pub viewport: (f32, f32),
    pub world_ubo: PerFrame<CpuBuffer<WorldUBO>, 2>,

    pub camera: Option<hikari_core::Entity>,
    pub directional_light: Option<hikari_core::Entity>
}
