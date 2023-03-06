use std::sync::Arc;

use hikari_asset::Asset;
use hikari_render::{SampledImage};

use crate::config::*;

pub struct TextureCube {
    image: SampledImage,
    config: TextureConfig,
}

impl TextureCube {
    pub fn new<T: Copy>(
        device: &Arc<hikari_render::Device>,
        faces: &[T],
        width: u32,
        height: u32,
        config: TextureConfig,
    ) -> Result<Self, anyhow::Error> {
        let image = SampledImage::with_layers(device, &faces, width, height, 1, 6, config.into_image_config_cube(width, height)?)?;

        Ok(Self {
            image,
            config
        })
    }
    pub fn with_dimensions(
        device: &Arc<hikari_render::Device>,
        width: u32,
        height: u32,
        config: TextureConfig,
    ) -> Result<Self, anyhow::Error> {
        let image_config = config.into_image_config_cube(width, height)?;
        let image = SampledImage::with_dimensions(device, width, height, 1, 6, image_config)?;

        Ok(Self {
            image,
            config
        })
    }
    pub fn from_parts(image: SampledImage, config: TextureConfig) -> Self {
        Self {
            image,
            config
        }
    } 
    pub fn raw(&self) -> &SampledImage {
        &self.image
    }
    pub fn width(&self) -> u32 {
        self.image.width()
    }
    pub fn height(&self) -> u32 {
        self.image.height()
    }
    pub fn config(&self) -> &TextureConfig {
        &self.config
    }
}

impl Asset for TextureCube {
    type Settings = TextureConfig;
}