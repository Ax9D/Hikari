use std::sync::Arc;

use crate::{Settings, WorldUBO, common::PerInstanceData, instancing::MeshInstancer};
use hikari_render::{Device, SampledImage, RingBuffer};

pub const MAX_ENTITIES: usize = 10_000;
pub const SCENE_SET_ID: u32 = 1;
pub struct RenderResources {
    device: Arc<Device>,
    pub settings: Settings,
    pub viewport: (f32, f32),
    pub world_ubo: RingBuffer<WorldUBO>,
    pub instance_ssbo: RingBuffer<PerInstanceData>,
    pub mesh_instancer: MeshInstancer,
    pub hi_z_images: Vec<SampledImage>,

    pub camera: Option<hikari_core::Entity>,
    pub directional_light: Option<hikari_core::Entity>,
}

impl RenderResources {
    pub fn new(device: &Arc<Device>, width: u32, height: u32, settings: Settings) -> anyhow::Result<Self> {
        Ok(RenderResources {
            device: device.clone(),
            settings,
            viewport: (width as f32, height as f32),
            camera: None,
            directional_light: None,
            world_ubo: hikari_render::create_uniform_buffer(device, 1)?,
            instance_ssbo: hikari_render::create_storage_buffer(device, MAX_ENTITIES)?,
            mesh_instancer: MeshInstancer::new(),
            hi_z_images: crate::passes::shadow::create_hi_z_images(device, width, height)?,
        })
    }
    pub fn on_resize(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        self.hi_z_images = crate::passes::shadow::create_hi_z_images(&self.device, width, height)?;

        Ok(())
    }
}