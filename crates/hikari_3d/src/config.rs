use hikari_render::{vk, ImageConfig};
use serde::{Deserialize, Serialize};

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

#[derive(Copy, Clone, Serialize, Deserialize, type_uuid::TypeUuid)]
#[serde(default)]
#[uuid = "304e0256-1f2c-4a71-94f6-6e858fa4d9be"]
pub struct TextureConfig {
    pub format: Format,
    pub filtering: FilterMode,
    pub wrap_x: WrapMode,
    pub wrap_y: WrapMode,
    pub aniso_level: f32,
    pub generate_mips: bool,
    pub max_mip_levels: u32,
}

impl Default for TextureConfig {
    fn default() -> Self {
        Self {
            format: Format::RGBA8,
            filtering: FilterMode::Linear,
            wrap_x: WrapMode::Repeat,
            wrap_y: WrapMode::Repeat,
            aniso_level: 8.0,
            generate_mips: true,
            max_mip_levels: 15,
        }
    }
}
impl TextureConfig {
    pub fn into_image_config_2d(&self, width: u32, height: u32) -> anyhow::Result<ImageConfig> {
        let format = match self.format {
            //Format::RGB8 => vk::Format::R8G8B8_SNORM,
            Format::RGBA8 => vk::Format::R8G8B8A8_UNORM,
            //Format::SRGB => vk::Format::R8G8B8_SRGB,
            Format::SRGBA => vk::Format::R8G8B8A8_SRGB,
            Format::RGBAFloat16 => vk::Format::R16G16B16A16_SFLOAT,
            Format::RGBAFloat32 => vk::Format::R32G32B32A32_SFLOAT,
        };
        let filtering = match self.filtering {
            FilterMode::Closest => vk::Filter::NEAREST,
            FilterMode::Linear => vk::Filter::LINEAR,
        };
        let wrap_x = match self.wrap_x {
            WrapMode::Clamp => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            WrapMode::Repeat => vk::SamplerAddressMode::REPEAT,
        };
        let wrap_y = match self.wrap_y {
            WrapMode::Clamp => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            WrapMode::Repeat => vk::SamplerAddressMode::REPEAT,
        };
        let wrap_z = vk::SamplerAddressMode::REPEAT;

        let mip_filtering = match self.filtering {
            FilterMode::Closest => vk::SamplerMipmapMode::NEAREST,
            FilterMode::Linear => vk::SamplerMipmapMode::LINEAR,
        };

        if self.generate_mips && self.max_mip_levels == 0 {
            return Err(anyhow::anyhow!("Max mip levels must be greater than 0"));
        }

        Ok(ImageConfig {
            format,
            filtering,
            wrap_x,
            wrap_y,
            wrap_z,
            sampler_reduction_mode: None,
            aniso_level: self.aniso_level,
            mip_levels: if self.generate_mips {
                TextureConfig::get_mip_count(width, height).min(self.max_mip_levels)
            } else {
                1
            },
            mip_filtering,
            usage: vk::ImageUsageFlags::SAMPLED,
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            image_view_type: vk::ImageViewType::TYPE_2D,
            initial_layout: vk::ImageLayout::UNDEFINED,
            host_readable: false,
        })
    }
    pub fn into_image_config_cube(&self, width: u32, height: u32) -> anyhow::Result<ImageConfig> {
        let format = match self.format {
            //Format::RGB8 => vk::Format::R8G8B8_SNORM,
            Format::RGBA8 => vk::Format::R8G8B8A8_UNORM,
            //Format::SRGB => vk::Format::R8G8B8_SRGB,
            Format::SRGBA => vk::Format::R8G8B8A8_SRGB,

            Format::RGBAFloat16 => vk::Format::R16G16B16A16_SFLOAT,
            Format::RGBAFloat32 => vk::Format::R32G32B32A32_SFLOAT,
        };
        let filtering = match self.filtering {
            FilterMode::Closest => vk::Filter::NEAREST,
            FilterMode::Linear => vk::Filter::LINEAR,
        };
        let wrap_x = match self.wrap_x {
            WrapMode::Clamp => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            WrapMode::Repeat => vk::SamplerAddressMode::REPEAT,
        };
        let wrap_y = match self.wrap_y {
            WrapMode::Clamp => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            WrapMode::Repeat => vk::SamplerAddressMode::REPEAT,
        };
        let wrap_z = vk::SamplerAddressMode::CLAMP_TO_EDGE;

        let mip_filtering = match self.filtering {
            FilterMode::Closest => vk::SamplerMipmapMode::NEAREST,
            FilterMode::Linear => vk::SamplerMipmapMode::LINEAR,
        };

        if self.generate_mips && self.max_mip_levels == 0 {
            return Err(anyhow::anyhow!("Max mip levels must be greater than 0"));
        }

        Ok(ImageConfig {
            format,
            filtering,
            wrap_x,
            wrap_y,
            wrap_z,
            sampler_reduction_mode: None,
            aniso_level: self.aniso_level,
            mip_levels: if self.generate_mips {
                TextureConfig::get_mip_count(width, height).min(self.max_mip_levels)
            } else {
                1
            },
            mip_filtering,
            usage: vk::ImageUsageFlags::SAMPLED,
            flags: vk::ImageCreateFlags::CUBE_COMPATIBLE,
            image_type: vk::ImageType::TYPE_2D,
            image_view_type: vk::ImageViewType::CUBE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            host_readable: false,
        })
    }
}
