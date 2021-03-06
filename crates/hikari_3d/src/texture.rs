use image::EncodableLayout;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum FilterMode {
    Closest,
    Linear,
}

impl Default for FilterMode {
    fn default() -> Self {
        Self::Linear
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Format {
    RGBA8,

    RGBAFloat16,
    RGBAFloat32,

    SRGBA,
}

impl Default for Format {
    fn default() -> Self {
        Self::RGBA8
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum WrapMode {
    Clamp,
    Repeat,
}

impl Default for WrapMode {
    fn default() -> Self {
        Self::Repeat
    }
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct TextureConfig {
    pub format: Format,
    pub filtering: FilterMode,
    pub wrap_x: WrapMode,
    pub wrap_y: WrapMode,
    pub aniso_level: f32,
    pub generate_mips: bool,
}
impl TextureConfig {
    pub fn get_mip_count(width: u32, height: u32) -> u32 {
        ((u32::max(width, height) as f32).log2().floor() + 1.0) as u32
    }
    // pub fn into_vk_config(&self, width: u32, height: u32) -> ImageConfig {
    //     ImageConfig {
    //         format: self.format.into_vk(),
    //         filtering: self.filtering.into_vk(),
    //         wrap_x: self.wrap_x.into_vk(),
    //         wrap_y: self.wrap_y.into_vk(),
    //         aniso_level: self.aniso_level,
    //         mip_levels: if self.generate_mips {
    //             Self::get_mip_count(width, height)
    //         } else {
    //             1
    //         },
    //         mip_filtering: self.filtering.into_vk_mip(),
    //         usage: vk::ImageUsageFlags::SAMPLED,
    //         image_type: vk::ImageType::TYPE_2D,
    //         host_readable: false,
    //     }
    // }
}

use std::sync::Arc;

use hikari_asset::{Asset, Load, LoadContext, MetaData};
use hikari_render::*;

pub fn into_vk_config(config: &TextureConfig, width: u32, height: u32) -> ImageConfig {
    let format = match config.format {
        //Format::RGB8 => vk::Format::R8G8B8_SNORM,
        Format::RGBA8 => vk::Format::R8G8B8A8_UNORM,
        //Format::SRGB => vk::Format::R8G8B8_SRGB,
        Format::SRGBA => vk::Format::R8G8B8A8_SRGB,
        Format::RGBAFloat16 => vk::Format::R16G16B16A16_SFLOAT,
        Format::RGBAFloat32 => vk::Format::R32G32B32A32_SFLOAT,
    };
    let filtering = match config.filtering {
        FilterMode::Closest => vk::Filter::NEAREST,
        FilterMode::Linear => vk::Filter::LINEAR,
    };
    let wrap_x = match config.wrap_x {
        WrapMode::Clamp => vk::SamplerAddressMode::CLAMP_TO_EDGE,
        WrapMode::Repeat => vk::SamplerAddressMode::REPEAT,
    };
    let wrap_y = match config.wrap_y {
        WrapMode::Clamp => vk::SamplerAddressMode::CLAMP_TO_EDGE,
        WrapMode::Repeat => vk::SamplerAddressMode::REPEAT,
    };
    let mip_filtering = match config.filtering {
        FilterMode::Closest => vk::SamplerMipmapMode::NEAREST,
        FilterMode::Linear => vk::SamplerMipmapMode::LINEAR,
    };

    ImageConfig {
        format,
        filtering,
        wrap_x,
        wrap_y,
        aniso_level: config.aniso_level,
        mip_levels: if config.generate_mips {
            TextureConfig::get_mip_count(width, height)
        } else {
            1
        },
        mip_filtering,
        usage: vk::ImageUsageFlags::SAMPLED,
        image_type: vk::ImageType::TYPE_2D,
        host_readable: false,
    }
}
pub struct Texture2D {
    image: SampledImage,
    config: TextureConfig,
}
impl Texture2D {
    pub fn new(
        device: &Arc<hikari_render::Device>,
        data: &[u8],
        width: u32,
        height: u32,
        config: TextureConfig,
    ) -> Result<Texture2D, anyhow::Error> {
        Ok(Self {
            image: SampledImage::with_rgba8(
                device,
                data,
                width,
                height,
                into_vk_config(&config, width, height),
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
    pub fn config(&self) -> &TextureConfig {
        &self.config
    }
}
pub struct TextureLoader {
    pub device: Arc<Device>,
}

impl Asset for Texture2D {
    const NAME: &'static str = "Texture2D";

    fn extensions<'a>() -> &'a [&'static str] {
        &["png", "jpeg", "jpg"]
    }
}
impl Load for Texture2D {
    type Loader = TextureLoader;
    type LoadSettings = TextureConfig;

    fn load(
        loader: &Self::Loader,
        bytes: &[u8],
        meta: &MetaData<Self>,
        _context: &mut LoadContext,
    ) -> Result<Self, hikari_asset::Error> {
        let image = image::load_from_memory(bytes)?;
        let image = image.to_rgba8();
        let data = image.as_bytes();
        let width = image.width();
        let height = image.height();

        Self::new(&loader.device, data, width, height, meta.settings)
    }
}
