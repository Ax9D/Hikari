use std::sync::Arc;

use ash::vk;
use gpu_allocator::{vulkan::Allocation, vulkan::AllocationCreateDesc, AllocationError};

pub mod sampled_image;
pub mod texture2d;

pub use sampled_image::ImageConfig;
pub use sampled_image::SampledImage;

pub use texture2d::Texture2D;

#[derive(Copy, Clone)]
pub enum FilterMode {
    Closest,
    Linear,
}
impl FilterMode {
    pub fn into_vk(&self) -> vk::Filter {
        match self {
            FilterMode::Closest => vk::Filter::NEAREST,
            FilterMode::Linear => vk::Filter::LINEAR,
        }
    }
    pub fn into_vk_mip(&self) -> vk::SamplerMipmapMode {
        match self {
            FilterMode::Closest => vk::SamplerMipmapMode::NEAREST,
            FilterMode::Linear => vk::SamplerMipmapMode::LINEAR,
        }
    }
}
impl Default for FilterMode {
    fn default() -> Self {
        Self::Linear
    }
}

#[derive(Copy, Clone)]
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

impl Format {
    pub fn into_vk(&self) -> vk::Format {
        match self {
            //Format::RGB8 => vk::Format::R8G8B8_SNORM,
            Format::RGBA8 => vk::Format::R8G8B8A8_SNORM,
            //Format::SRGB => vk::Format::R8G8B8_SRGB,
            Format::SRGBA => vk::Format::R8G8B8A8_SRGB,
            Format::RGBAFloat16 => vk::Format::R16G16B16A16_SFLOAT,
            Format::RGBAFloat32 => vk::Format::R32G32B32A32_SFLOAT,
        }
    }
}

#[derive(Copy, Clone)]
pub enum WrapMode {
    Clamp,
    Repeat,
}
impl WrapMode {
    pub fn into_vk(&self) -> vk::SamplerAddressMode {
        match self {
            WrapMode::Clamp => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            WrapMode::Repeat => vk::SamplerAddressMode::REPEAT,
        }
    }
}

impl Default for WrapMode {
    fn default() -> Self {
        Self::Repeat
    }
}

#[derive(Copy, Clone, Default)]
pub struct TextureConfig {
    pub format: super::Format,
    pub filtering: super::FilterMode,
    pub wrap_x: super::WrapMode,
    pub wrap_y: super::WrapMode,
    pub aniso_level: u8,
    pub generate_mips: bool,
}
impl TextureConfig {
    fn get_mip_count(width: u32, height: u32) -> u32 {
        ((u32::max(width, height) as f32).log2().floor() + 1.0) as u32
    }
    pub fn into_vk_config(&self, width: u32, height: u32) -> ImageConfig {
        ImageConfig {
            format: self.format.into_vk(),
            filtering: self.filtering.into_vk(),
            wrap_x: self.wrap_x.into_vk(),
            wrap_y: self.wrap_y.into_vk(),
            aniso_level: self.aniso_level,
            mip_levels: if self.generate_mips {
                Self::get_mip_count(width, height)
            } else {
                1
            },
            mip_filtering: self.filtering.into_vk_mip(),
            usage: vk::ImageUsageFlags::SAMPLED,
            image_type: vk::ImageType::TYPE_2D,
            host_readable: true,
        }
    }
}

pub fn create_image(
    device: &Arc<crate::Device>,
    create_info: &vk::ImageCreateInfo,
    location: gpu_allocator::MemoryLocation,
) -> Result<(vk::Image, Allocation), Box<dyn std::error::Error>> {
    unsafe {
        let image = device.raw().create_image(create_info, None)?;
        let requirements = device.raw().get_image_memory_requirements(image);
        let allocation = device.allocate_memory(AllocationCreateDesc {
            name: "image",
            requirements,
            location,
            linear: false,
        })?;

        device
            .raw()
            .bind_image_memory(image, allocation.memory(), allocation.offset())?;

        Ok((image, allocation))
    }
}
pub fn delete_image(
    device: &Arc<crate::Device>,
    image: vk::Image,
    allocation: Allocation,
) -> Result<(), AllocationError> {
    unsafe {
        device.raw().destroy_image(image, None);
    }
    device.free_memory(allocation)
}
