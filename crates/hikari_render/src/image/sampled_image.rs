use std::{sync::Arc, ops::Range, collections::HashMap};

use ash::{prelude::VkResult, vk};
use gpu_allocator::vulkan::Allocation;
use parking_lot::Mutex;

use crate::buffer::Buffer;

fn format_size(format: vk::Format) -> u32 {
    match format {
        vk::Format::R8G8B8A8_SINT
        | vk::Format::R8G8B8A8_SNORM
        | vk::Format::R8G8B8A8_SRGB
        | vk::Format::R8G8B8A8_UNORM => 4,
        vk::Format::R16G16B16A16_SINT
        | vk::Format::R16G16B16A16_SNORM
        | vk::Format::R16G16B16A16_SFLOAT => 2 * 4,
        vk::Format::R32G32B32A32_SINT
        | vk::Format::R32G32B32A32_SFLOAT
        | vk::Format::R32G32B32A32_UINT => 4 * 4,
        vk::Format::R32G32_SFLOAT => 2 * 4,
        vk::Format::D16_UNORM => 2,
        vk::Format::D16_UNORM_S8_UINT => 3,
        vk::Format::D24_UNORM_S8_UINT => 4,
        vk::Format::D32_SFLOAT => 4,
        vk::Format::D32_SFLOAT_S8_UINT => 5,
        _ => todo!(),
    }
}
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
            host_readable: false,
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
            host_readable: false,
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
            host_readable: false,
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
            host_readable: false,
        }
    }
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ImageViewDesc {
    pub view_type: vk::ImageViewType,
    pub mip_range: Range<u32>,
    pub layer_range: Range<u32>
}
/// An Image that can be sampled in shaders
/// An ImageView is generated for each mip level automatically
pub struct SampledImage {
    device: Arc<crate::Device>,
    allocation: Option<Allocation>,
    image: vk::Image,
    basic_image_views: Vec<vk::ImageView>,
    arbitrary_image_views: Mutex<HashMap<ImageViewDesc, vk::ImageView, crate::util::BuildHasher>>,
    sampler: vk::Sampler,
    config: ImageConfig,
    width: u32,
    height: u32,
    depth: u32,
    layers: u32,

    download_buffer: Option<crate::buffer::CpuBuffer<u8>>,
}

pub(crate) fn format_to_aspect_flags(format: vk::Format) -> vk::ImageAspectFlags {
    match format {
        vk::Format::D16_UNORM | vk::Format::D32_SFLOAT => vk::ImageAspectFlags::DEPTH,
        vk::Format::D16_UNORM_S8_UINT
        | vk::Format::D24_UNORM_S8_UINT
        | vk::Format::D32_SFLOAT_S8_UINT => {
            vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
        }
        _ => vk::ImageAspectFlags::COLOR,
    }
}

impl SampledImage {
    fn create_sampler(
        device: &Arc<crate::Device>,
        vkconfig: &ImageConfig,
    ) -> VkResult<vk::Sampler> {
        //TODO: use a sampler cache
        let mut create_info = vk::SamplerCreateInfo::builder()
            .min_filter(vkconfig.filtering)
            .mag_filter(vkconfig.filtering)
            .mipmap_mode(vkconfig.mip_filtering)
            .compare_op(vk::CompareOp::NEVER)
            .min_lod(0.0)
            .max_lod(0.0) // initial value
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .anisotropy_enable(false) // initial value
            .address_mode_u(vkconfig.wrap_x)
            .address_mode_v(vkconfig.wrap_y)
            .address_mode_w(vkconfig.wrap_z)
            .max_anisotropy(0.0);

        if vkconfig.mip_levels > 1 {
            create_info.max_lod = vkconfig.mip_levels as f32;
        }

        if vkconfig.aniso_level > 0.0
            && device.is_feature_supported(crate::device::Features::SAMPLER_ANISOTROPY)
        {
            create_info.max_anisotropy = vkconfig.aniso_level;
            create_info.anisotropy_enable = vk::TRUE;
        }

        let mut reduce_info = vk::SamplerReductionModeCreateInfo::builder();

        let create_info = if let Some(reduction_mode) = vkconfig.sampler_reduction_mode {
            reduce_info.reduction_mode = reduction_mode;
            create_info.push_next(&mut reduce_info)
        } else {
            create_info
        };

        unsafe { device.raw().create_sampler(&create_info, None) }
    }
    fn create_view(device: &Arc<crate::Device>,
        image: vk::Image,
        create_info: &vk::ImageViewCreateInfo) -> VkResult<vk::ImageView> {
        let view = unsafe { device.raw().create_image_view(create_info, None)? };

        Ok(view)
    }
    fn create_views(
        device: &Arc<crate::Device>,
        image: vk::Image,
        vkconfig: &ImageConfig,
    ) -> VkResult<Vec<vk::ImageView>> {
        let mut views = Vec::new();

        for mip_level in 0..vkconfig.mip_levels {
            let create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .format(vkconfig.format)
            .view_type(vkconfig.image_view_type)
            .subresource_range(
                *vk::ImageSubresourceRange::builder()
                    .aspect_mask(format_to_aspect_flags(vkconfig.format))
                    .base_mip_level(mip_level)
                    .level_count(vk::REMAINING_MIP_LEVELS)
                    .base_array_layer(0)
                    .layer_count(vk::REMAINING_ARRAY_LAYERS),
            )
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            });
            let view = Self::create_view(device, image, &create_info)?;
            views.push(view);
        }

        Ok(views)
    }
    fn create_image_with_sampler_and_views(
        device: &Arc<crate::Device>,
        width: u32,
        height: u32,
        depth: u32,
        layers: u32,
        vkconfig: &ImageConfig,
    ) -> anyhow::Result<(
        vk::Image,
        gpu_allocator::vulkan::Allocation,
        vk::Sampler,
        Vec<vk::ImageView>,
    )> {
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vkconfig.image_type)
            .format(vkconfig.format)
            .mip_levels(vkconfig.mip_levels)
            .array_layers(layers)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .sharing_mode(vk::SharingMode::EXCLUSIVE /**/)
            .queue_family_indices(&[0])
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .extent(vk::Extent3D {
                width,
                height,
                depth,
            })
            .usage(
                vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::TRANSFER_SRC
                    | vkconfig.usage,
            )
            .flags(vkconfig.flags);

        let (image, allocation) = crate::image::create_image(
            device,
            &image_create_info,
            gpu_allocator::MemoryLocation::GpuOnly,
        )?;

        if !(vkconfig.initial_layout == vk::ImageLayout::UNDEFINED
            || vkconfig.initial_layout == vk::ImageLayout::PREINITIALIZED)
        {
            let device_raw = device.raw();
            unsafe {
                device.submit_commands_immediate(|cmd| {
                    crate::barrier::image_memory_barrier(
                        device_raw,
                        cmd,
                        image,
                        vk::ImageSubresourceRange {
                            aspect_mask: format_to_aspect_flags(vkconfig.format),
                            base_mip_level: 0,
                            level_count: vk::REMAINING_MIP_LEVELS,
                            base_array_layer: 0,
                            layer_count: vk::REMAINING_ARRAY_LAYERS,
                        },
                        vk::AccessFlags::empty(),
                        vk::AccessFlags::empty(),
                        vk::ImageLayout::UNDEFINED,
                        vkconfig.initial_layout,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                    );

                    Ok(())
                })?;
            }
        }

        let sampler = Self::create_sampler(device, vkconfig)?;
        let image_views = Self::create_views(device, image, vkconfig)?;

        Ok((image, allocation, sampler, image_views))
    }
    /// Creates an empty image of the specified width and height
    pub fn with_dimensions(
        device: &Arc<crate::Device>,
        width: u32,
        height: u32,
        depth: u32,
        layers: u32,
        vkconfig: ImageConfig,
    ) -> anyhow::Result<Self> {
        let (image, allocation, sampler, image_views) =
            Self::create_image_with_sampler_and_views(device, width, height, depth, layers, &vkconfig)?;

        let download_buffer = if vkconfig.host_readable {
            Some(crate::buffer::CpuBuffer::new(
                device,
                (width * height * depth * format_size(vkconfig.format)) as usize,
                vk::BufferUsageFlags::TRANSFER_DST,
                gpu_allocator::MemoryLocation::GpuToCpu,
            )?)
        } else {
            None
        };

        Ok(Self {
            device: device.clone(),
            image,
            allocation: Some(allocation),
            sampler,
            basic_image_views: image_views,
            arbitrary_image_views: Mutex::new(Default::default()),
            config: vkconfig,
            width,
            height,
            depth,
            layers,
            download_buffer,
        })
    }
    /// Creates an image with the specified pixel data, width, height and depth
    pub fn with_data<T: Copy>(
        device: &Arc<crate::Device>,
        data: &[T],
        width: u32,
        height: u32,
        depth: u32,
        vkconfig: ImageConfig,
    ) -> anyhow::Result<Self> {
        Self::with_layers(device, data, width, height, depth, 1, vkconfig)
    }
    pub fn with_layers<T: Copy>(
        device: &Arc<crate::Device>,
        data: &[T],
        width: u32,
        height: u32,
        depth: u32,
        layers: u32,
        mut vkconfig: ImageConfig,
    ) -> anyhow::Result<Self> {
        vkconfig.usage |= vk::ImageUsageFlags::TRANSFER_DST;

        if vkconfig.host_readable {
            vkconfig.usage |= vk::ImageUsageFlags::TRANSFER_SRC;
        }

        let image_buffer_max_size =
            (width * height * depth * layers) as usize * format_size(vkconfig.format) as usize;

        let data = unsafe {
                std::slice::from_raw_parts::<u8>(data.as_ptr() as *const u8, data.len() * std::mem::size_of::<T>() / std::mem::size_of::<u8>())
        };
        //FIX ME: This is probably wrong?, Dont assume format sizes
        if data.len() != image_buffer_max_size {
            return Err(anyhow::anyhow!(
                "Cannot create gpu image, data size {} bytes doesn't match expected size {} bytes, format is {:?}",
                data.len(),
                image_buffer_max_size,
                vkconfig.format
            ));
        }
        let (image, allocation, sampler, image_views) =
            Self::create_image_with_sampler_and_views(device, width, height, depth, layers, &vkconfig)?;

        let subresource_range = *vk::ImageSubresourceRange::builder()
            .aspect_mask(format_to_aspect_flags(vkconfig.format))
            .base_mip_level(0)
            .level_count(vkconfig.mip_levels)
            .layer_count(layers);

        let mut staging_buffer = crate::buffer::CpuBuffer::new(
            device,
            data.len(),
            vk::BufferUsageFlags::TRANSFER_SRC,
            gpu_allocator::MemoryLocation::CpuToGpu,
        )?;

        unsafe {
            let slice = staging_buffer.mapped_slice_mut();

            slice.copy_from_slice(data);
        }
        unsafe {
            hikari_dev::profile_scope!("Image Upload");
            device.submit_commands_immediate(|cmd| {
                let device = device.raw();

                crate::barrier::image_memory_barrier(
                    device,
                    cmd,
                    image,
                    subresource_range,
                    vk::AccessFlags::empty(),
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::ImageLayout::UNDEFINED,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                );

                let buffer_copy_region = [*vk::BufferImageCopy::builder()
                    .image_subresource(
                        *vk::ImageSubresourceLayers::builder()
                            .aspect_mask(format_to_aspect_flags(vkconfig.format))
                            .mip_level(0)
                            .base_array_layer(0)
                            .layer_count(layers),
                    )
                    .image_extent(vk::Extent3D {
                        width,
                        height,
                        depth,
                    })];

                device.cmd_copy_buffer_to_image(
                    cmd,
                    staging_buffer.buffer(),
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &buffer_copy_region,
                );
                Ok(())
            })?;

            device.submit_commands_immediate(|cmd| {
                if vkconfig.mip_levels > 1 {
                    hikari_dev::profile_scope!("Generate mips");

                    crate::barrier::image_memory_barrier(
                        device.raw(),
                        cmd,
                        image,
                        subresource_range,
                        vk::AccessFlags::TRANSFER_WRITE,
                        vk::AccessFlags::TRANSFER_READ,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::TRANSFER,
                    );

                    Self::generate_mips_(device.raw(), cmd, image, width, height, depth, layers, &vkconfig);

                    crate::barrier::image_memory_barrier(
                        device.raw(),
                        cmd,
                        image,
                        subresource_range,
                        vk::AccessFlags::TRANSFER_READ,
                        vk::AccessFlags::MEMORY_READ,
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                    );
                }
                else {
                    crate::barrier::image_memory_barrier(
                        device.raw(),
                        cmd,
                        image,
                        subresource_range,
                        vk::AccessFlags::TRANSFER_WRITE,
                        vk::AccessFlags::SHADER_READ,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                    );
            }

                Ok(())
            })?;
        }

        let download_buffer = if vkconfig.host_readable {
            Some(crate::buffer::CpuBuffer::new(
                device,
                data.len(),
                vk::BufferUsageFlags::TRANSFER_DST,
                gpu_allocator::MemoryLocation::GpuToCpu,
            )?)
        } else {
            None
        };

        Ok(Self {
            device: device.clone(),
            image,
            basic_image_views: image_views,
            arbitrary_image_views: Mutex::new(Default::default()),
            sampler,
            allocation: Some(allocation),
            width,
            height,
            depth,
            layers,
            config: vkconfig,
            download_buffer,
        })
    }
    /// Assumes that image is in TRANSFER_SRC_OPTIMAL layout
    pub fn generate_mips(&self, cmd: vk::CommandBuffer) {
        Self::generate_mips_(self.device.raw(), cmd, self.image, self.width, self.height, self.depth, self.layers, &self.config)
    }
    fn generate_mips_(
        device: &ash::Device,
        cmd: vk::CommandBuffer,
        image: vk::Image,
        width: u32,
        height: u32,
        depth: u32,
        layers: u32,
        config: &ImageConfig,
    ) {
        let levels = config.mip_levels;

        unsafe {
            for layer in 0..layers {

                let mut mip_width = width as i32;
                let mut mip_height = height as i32;
                let mut mip_depth = depth as i32;

                for level in 1..levels {
                    let next_mip_width = if mip_width > 1 {
                        mip_width / 2
                    } else {
                        mip_width
                    };
                    let next_mip_height = if mip_height > 1 {
                        mip_height / 2
                    } else {
                        mip_height
                    };
                    let next_mip_depth = if mip_depth > 1 {
                        mip_depth / 2
                    } else {
                        mip_depth
                    };

                    let image_blit = [*vk::ImageBlit::builder()
                        .src_subresource(
                            *vk::ImageSubresourceLayers::builder()
                                .aspect_mask(format_to_aspect_flags(config.format))
                                .layer_count(1)
                                .mip_level(level - 1)
                                .base_array_layer(layer),
                        )
                        .src_offsets([
                            vk::Offset3D { x: 0, y: 0, z: 0 },
                            vk::Offset3D {
                                x: mip_width,
                                y: mip_height,
                                z: mip_depth,
                            },
                        ])
                        .dst_subresource(
                            *vk::ImageSubresourceLayers::builder()
                                .aspect_mask(format_to_aspect_flags(config.format))
                                .layer_count(1)
                                .mip_level(level)
                                .base_array_layer(layer),
                        )
                        .dst_offsets([
                            vk::Offset3D { x: 0, y: 0, z: 0 },
                            vk::Offset3D {
                                x: next_mip_width,
                                y: next_mip_height,
                                z: next_mip_depth,
                            },
                        ])];

                    let mip_sub_range = *vk::ImageSubresourceRange::builder()
                        .aspect_mask(format_to_aspect_flags(config.format))
                        .base_mip_level(level)
                        .level_count(1)
                        .base_array_layer(layer)
                        .layer_count(1);

                    crate::barrier::image_memory_barrier(
                        device,
                        cmd,
                        image,
                        mip_sub_range,
                        vk::AccessFlags::empty(),
                        vk::AccessFlags::TRANSFER_WRITE,
                        vk::ImageLayout::UNDEFINED,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::TRANSFER,
                    );

                    device.cmd_blit_image(
                        cmd,
                        image,
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                        image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &image_blit,
                        config.filtering,
                    );

                    crate::barrier::image_memory_barrier(
                        device,
                        cmd,
                        image,
                        mip_sub_range,
                        vk::AccessFlags::TRANSFER_WRITE,
                        vk::AccessFlags::TRANSFER_READ,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::TRANSFER,
                    );

                    mip_width = next_mip_width;
                    mip_height = next_mip_height;
                    mip_depth = next_mip_depth;
                }
            }
        }
    }
    pub fn image(&self) -> vk::Image {
        self.image
    }
    pub fn image_view(&self, mip_level: usize) -> Option<vk::ImageView> {
        self.basic_image_views.get(mip_level).copied()
    }
    pub fn custom_image_view(&self, view_desc: ImageViewDesc) -> vk::ImageView {
        let mut views = self.arbitrary_image_views.lock();
        let view = views.entry(view_desc.clone()).or_insert_with(|| {
            let base_mip_level = view_desc.mip_range.start;
            let level_count = view_desc.mip_range.len() as u32;
            assert!(level_count <= self.config.mip_levels);

            let base_array_layer = view_desc.layer_range.start;
            let layer_count = view_desc.layer_range.len() as u32;
            assert!(layer_count <= self.layers);

            let create_info = vk::ImageViewCreateInfo::builder()
            .image(self.image)
            .format(self.config.format)
            .view_type(view_desc.view_type)
            .subresource_range(
                *vk::ImageSubresourceRange::builder()
                    .aspect_mask(format_to_aspect_flags(self.config.format))
                    .base_mip_level(base_mip_level)
                    .level_count(level_count)
                    .base_array_layer(base_array_layer)
                    .layer_count(layer_count),
            )
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            });
            Self::create_view(&self.device, self.image, &create_info).unwrap()
        });

        *view
    }
    pub fn sampler(&self) -> vk::Sampler {
        self.sampler
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn depth(&self) -> u32 {
        self.depth
    }
    pub fn layers(&self) -> u32 {
        self.layers
    }
    pub fn config(&self) -> &ImageConfig {
        &self.config
    }

    /// Copies the image from the GPU to the Host; the read is not synchronized on the GPU, the caller must ensure the image is not being used on the GPU
    /// Returns a slice to the downloaded data if the image was created with `host_readable` set to `true` in the `ImageConfig`
    /// Otherwise returns `None`
    pub fn download(&self, mip_level: u32, layout: vk::ImageLayout) -> Option<&[u8]> {
        if let Some(ref download_buffer) = self.download_buffer {
            assert!(mip_level > 0);

            unsafe {
                self.device
                    .submit_commands_immediate(|cmd| {
                        let device = self.device.raw();

                        let subresource_range = *vk::ImageSubresourceRange::builder()
                            .layer_count(1)
                            .level_count(mip_level)
                            .aspect_mask(format_to_aspect_flags(self.config.format));

                        crate::barrier::image_memory_barrier(
                            device,
                            cmd,
                            self.image,
                            subresource_range,
                            vk::AccessFlags::empty(),
                            vk::AccessFlags::TRANSFER_READ,
                            vk::ImageLayout::UNDEFINED,
                            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                            vk::PipelineStageFlags::TRANSFER,
                            vk::PipelineStageFlags::TRANSFER,
                        );

                        let regions = [*vk::BufferImageCopy::builder()
                            .buffer_offset(0)
                            .buffer_row_length(0)
                            .buffer_image_height(0)
                            .image_subresource(
                                *vk::ImageSubresourceLayers::builder()
                                    .mip_level(0)
                                    .aspect_mask(format_to_aspect_flags(self.config.format))
                                    .base_array_layer(0)
                                    .layer_count(1),
                            )
                            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                            .image_extent(vk::Extent3D {
                                width: self.width,
                                height: self.height,
                                depth: self.depth,
                            })];

                        device.cmd_copy_image_to_buffer(
                            cmd,
                            self.image,
                            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                            download_buffer.buffer(),
                            &regions,
                        );

                        crate::barrier::image_memory_barrier(
                            device,
                            cmd,
                            self.image,
                            subresource_range,
                            vk::AccessFlags::TRANSFER_READ,
                            vk::AccessFlags::empty(),
                            vk::ImageLayout::UNDEFINED,
                            layout,
                            vk::PipelineStageFlags::TRANSFER,
                            vk::PipelineStageFlags::TRANSFER,
                        );

                        Ok(())
                    })
                    .ok()?
            }
            let slice = download_buffer.mapped_slice();
            return Some(slice);
        }

        None
    }
}

impl Drop for SampledImage {
    fn drop(&mut self) {
        use crate::delete::DeleteRequest;
        let deleter = self.device.deleter();
        deleter.request_delete(DeleteRequest::VkSampler(self.sampler));

        for image_view in self.basic_image_views.drain(..) {
            deleter.request_delete(DeleteRequest::VkImageView(image_view));
        }
        for (_, image_view) in self.arbitrary_image_views.lock().drain() {
            deleter.request_delete(DeleteRequest::VkImageView(image_view));
        }
        let allocation = self.allocation.take().unwrap();
        deleter.request_delete(DeleteRequest::VkImage(self.image, allocation));
    }
}