use std::sync::Arc;

use ash::vk;

/// Specifies what the properties of the Image should be
/// If `host_readable` is set to true, the contents of the image can be copied back to the host
#[derive(Copy, Clone, Debug)]
pub struct ImageConfig {
    pub format: vk::Format,
    pub filtering: vk::Filter,
    pub wrap_x: vk::SamplerAddressMode,
    pub wrap_y: vk::SamplerAddressMode,
    pub wrap_z: vk::SamplerAddressMode,
    pub sampler_reduction_mode: Option<vk::SamplerReductionMode>,
    pub aniso_level: f32,
    pub mip_levels: u32,
    pub mip_filtering: vk::SamplerMipmapMode,
    pub usage: vk::ImageUsageFlags,
    pub flags: vk::ImageCreateFlags,
    pub image_type: vk::ImageType,
    pub image_view_type: vk::ImageViewType,
    pub initial_layout: vk::ImageLayout,
    pub host_readable: bool,
}


impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            format: vk::Format::R8G8B8A8_UNORM,
            filtering: vk::Filter::LINEAR,
            wrap_x: vk::SamplerAddressMode::REPEAT,
            wrap_y: vk::SamplerAddressMode::REPEAT,
            wrap_z: vk::SamplerAddressMode::REPEAT,
            sampler_reduction_mode: None,
            aniso_level: 0.0,
            mip_levels: 1,
            mip_filtering: vk::SamplerMipmapMode::LINEAR,
            usage: vk::ImageUsageFlags::SAMPLED,
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            image_view_type: vk::ImageViewType::TYPE_2D,
            initial_layout: vk::ImageLayout::UNDEFINED,
            host_readable: false,
        }
    }
}

impl ImageConfig {
    pub fn cubemap() -> Self {
        Self {
            usage: vk::ImageUsageFlags::SAMPLED,
            flags: vk::ImageCreateFlags::CUBE_COMPATIBLE,
            image_type: vk::ImageType::TYPE_2D,
            image_view_type: vk::ImageViewType::CUBE,
            ..Default::default()
        }
    }
    /// Creates a config for a 2D color attachment with a single mip level, linear filtering with
    /// usage flags `vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED`
    /// of format `R8G8B8A8_UNORM`
    pub fn color2d_attachment() -> Self {
        Self {
            format: vk::Format::R8G8B8A8_UNORM,
            filtering: vk::Filter::LINEAR,
            wrap_x: vk::SamplerAddressMode::REPEAT,
            wrap_y: vk::SamplerAddressMode::REPEAT,
            wrap_z: vk::SamplerAddressMode::REPEAT,
            sampler_reduction_mode: None,
            aniso_level: 0.0,
            mip_levels: 1,
            mip_filtering: vk::SamplerMipmapMode::LINEAR,
            usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            image_view_type: vk::ImageViewType::TYPE_2D,
            initial_layout: vk::ImageLayout::UNDEFINED,
            ..Default::default()
        }
    }
    pub fn color3d_attachment() -> Self {
        Self {
            format: vk::Format::R8G8B8A8_UNORM,
            filtering: vk::Filter::LINEAR,
            wrap_x: vk::SamplerAddressMode::REPEAT,
            wrap_y: vk::SamplerAddressMode::REPEAT,
            wrap_z: vk::SamplerAddressMode::REPEAT,
            sampler_reduction_mode: None,
            aniso_level: 0.0,
            mip_levels: 1,
            mip_filtering: vk::SamplerMipmapMode::LINEAR,
            usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_3D,
            image_view_type: vk::ImageViewType::TYPE_3D,
            initial_layout: vk::ImageLayout::UNDEFINED,
            ..Default::default()
        }
    }
    /// Creates a config for a depth stencil attachment, linear filtering with
    /// usage flags `vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED`
    /// A supported depth stencil format is picked automatically
    pub fn depth_stencil_attachment(device: &Arc<crate::Device>) -> Self {
        Self {
            format: device.supported_depth_stencil_format(),
            filtering: vk::Filter::NEAREST,
            wrap_x: vk::SamplerAddressMode::REPEAT,
            wrap_y: vk::SamplerAddressMode::REPEAT,
            wrap_z: vk::SamplerAddressMode::REPEAT,
            sampler_reduction_mode: None,
            aniso_level: 0.0,
            mip_levels: 1,
            mip_filtering: vk::SamplerMipmapMode::NEAREST,
            usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            image_view_type: vk::ImageViewType::TYPE_2D,
            initial_layout: vk::ImageLayout::UNDEFINED,
            ..Default::default()
        }
    }
    /// Creates a config for a depth only attachment, linear filtering with
    /// usage flags `vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED`
    /// A supported depth only format is picked automatically
    pub fn depth_only_attachment(device: &Arc<crate::Device>) -> Self {
        Self {
            format: device.supported_depth_only_format(),
            filtering: vk::Filter::NEAREST,
            wrap_x: vk::SamplerAddressMode::REPEAT,
            wrap_y: vk::SamplerAddressMode::REPEAT,
            wrap_z: vk::SamplerAddressMode::REPEAT,
            sampler_reduction_mode: None,
            aniso_level: 0.0,
            mip_levels: 1,
            mip_filtering: vk::SamplerMipmapMode::NEAREST,
            usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            image_view_type: vk::ImageViewType::TYPE_2D,
            initial_layout: vk::ImageLayout::UNDEFINED,
            ..Default::default()
        }
    }
}