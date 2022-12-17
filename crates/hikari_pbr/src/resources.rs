use std::sync::Arc;

use crate::{Settings, WorldUBO};
use hikari_render::{buffer::UniformBuffer, Device, SampledImage};

pub struct RenderResources {
    device: Arc<Device>,
    pub settings: Settings,
    pub viewport: (f32, f32),
    pub world_ubo: UniformBuffer<WorldUBO>,
    pub hi_z_images: Vec<SampledImage>,

    pub camera: Option<hikari_core::Entity>,
    pub directional_light: Option<hikari_core::Entity>,
}

impl RenderResources {
    pub fn new(device: &Arc<Device>, width: u32, height: u32) -> anyhow::Result<Self> {
        Ok(RenderResources {
            device: device.clone(),
            settings: Settings::new(),
            viewport: (width as f32, height as f32),
            camera: None,
            directional_light: None,
            world_ubo: UniformBuffer::<WorldUBO>::new(device, 1)?,
            hi_z_images: crate::passes::shadow::create_hi_z_images(device, width, height)?,
        })
    }
    pub fn on_resize(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        self.hi_z_images = crate::passes::shadow::create_hi_z_images(&self.device, width, height)?;

        Ok(())
    }
}
