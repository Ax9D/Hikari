use std::sync::Arc;

use super::sampled_image::SampledImage;

pub struct Texture2D {
    image: SampledImage,
    config: super::TextureConfig,
}
impl Texture2D {
    pub fn new(
        device: &Arc<crate::Device>,
        data: &[u8],
        width: u32,
        height: u32,
        config: super::TextureConfig,
    ) -> Result<Texture2D, Box<dyn std::error::Error>> {
        Ok(Self {
            image: SampledImage::with_data(
                device,
                data,
                width,
                height,
                config.into_vk_config(width, height),
            )?,
            config,
        })
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
    pub fn config(&self) -> &super::TextureConfig {
        &self.config
    }
}

pub trait Texture {}
