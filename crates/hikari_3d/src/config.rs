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

#[derive(Copy, Clone, Default, Serialize, Deserialize, type_uuid::TypeUuid)]
#[uuid = "304e0256-1f2c-4a71-94f6-6e858fa4d9be"]
pub struct TextureConfig {
    pub format: Format,
    pub filtering: FilterMode,
    pub wrap_x: WrapMode,
    pub wrap_y: WrapMode,
    pub aniso_level: f32,
    pub generate_mips: bool,
}