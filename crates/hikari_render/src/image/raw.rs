use std::{collections::HashMap};

use ash::{prelude::VkResult, vk};
use gpu_allocator::vulkan::Allocation;
use parking_lot::{Mutex};

use crate::{ImageViewDesc};

use crate::SamplerCreateInfo;
use crate::ImageConfig;

pub fn format_size(format: vk::Format) -> u32 {
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

pub(crate) struct RawSampledImage {
    pub image: vk::Image,
    pub allocation: Option<Allocation>,
    pub basic_image_views: Vec<vk::ImageView>,
    pub shader_resource_views: Vec<usize>,
    pub render_target_views: Vec<usize>,
    pub arbitrary_image_views: Mutex<HashMap<ImageViewDesc, vk::ImageView, crate::util::BuildHasher>>,
    pub sampler: vk::Sampler,
    pub config: ImageConfig,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub layers: u32,
}
impl RawSampledImage {
    pub fn delete_now(&mut self, device: &crate::Device) {

        for image_view in self.basic_image_views.drain(..) {
            crate::image::view::delete_view(device, image_view);
        }

        for (_, image_view) in self.arbitrary_image_views.lock().drain() {
            crate::image::view::delete_view(device, image_view);
        }

        let allocation = self.allocation.take().unwrap();
        crate::image::delete_image(device, self.image, allocation).unwrap();
    }
}

impl RawSampledImage {
    fn create_sampler(
        device: &crate::Device,
        vkconfig: &ImageConfig,
    ) -> VkResult<vk::Sampler> {
        let mut create_info = SamplerCreateInfo {
            mag_filter: vkconfig.filtering,
            min_filter: vkconfig.filtering,
            mipmap_mode: vkconfig.mip_filtering,
            address_mode_u: vkconfig.wrap_x,
            address_mode_v: vkconfig.wrap_y,
            address_mode_w: vkconfig.wrap_z,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 0.0, // initial value
            border_color: vk::BorderColor::FLOAT_OPAQUE_WHITE,
            anisotropy_enable: false, // initial_value
            sampler_reduction_mode: vkconfig.sampler_reduction_mode,
            ..Default::default()
        };

        if vkconfig.mip_levels > 1 {
            create_info.max_lod = vkconfig.mip_levels as f32;
        }

        if vkconfig.aniso_level > 0.0
            && device.is_feature_supported(crate::device::Features::SAMPLER_ANISOTROPY)
        {
            create_info.max_anisotropy = vkconfig.aniso_level;
            create_info.anisotropy_enable = true;
        }

        let sampler = device.cache().sampler().get_sampler(&create_info);

        Ok(sampler)
    }
    fn create_views(
        device: &crate::Device,
        image: vk::Image,
        image_config: &ImageConfig,
        views: &mut Vec<vk::ImageView>,
        shader_resource_views: &mut Vec<usize>,
        render_target_views: &mut Vec<usize>,
    ) -> VkResult<()> {
        if image_config.usage.contains(vk::ImageUsageFlags::SAMPLED) || 
           image_config.usage.contains(vk::ImageUsageFlags::STORAGE) {
            for mip_level in 0..image_config.mip_levels {
                let aspect = 

                if image_config.usage.contains(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT) {
                    vk::ImageAspectFlags::DEPTH
                } else {
                    format_to_aspect_flags(image_config.format)
                };
                
                let view_desc = ImageViewDesc {
                    view_type: image_config.image_view_type,
                    aspect,
                    mip_range: mip_level..image_config.mip_levels,
                    layer_range: 0..u32::MAX,
                };
                
                let view = crate::image::view::create_view(device, image, image_config, &view_desc)?;
                
                shader_resource_views.push(views.len());
                views.push(view);
            }
        }
        if image_config.usage.contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
            render_target_views.push(0);
        }
        
        if image_config.usage.contains(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT) {
            let view_desc = ImageViewDesc {
                view_type: image_config.image_view_type,
                aspect: format_to_aspect_flags(image_config.format),
                mip_range: 0..image_config.mip_levels,
                layer_range: 0..u32::MAX,
            };
            
            let view = crate::image::view::create_view(device, image, image_config, &view_desc)?;
            render_target_views.push(views.len());
            views.push(view);
        }
        assert!(render_target_views.len() <= 1);

        Ok(())
    }
    fn create_image_with_sampler_and_views(
        device: &crate::Device,
        width: u32,
        height: u32,
        depth: u32,
        layers: u32,
        vkconfig: &ImageConfig,
        views: &mut Vec<vk::ImageView>,
        shader_resource_views: &mut Vec<usize>,
        render_target_views: &mut Vec<usize>,
    ) -> anyhow::Result<(
        vk::Image,
        gpu_allocator::vulkan::Allocation,
        vk::Sampler,
    )> {
        let image_create_info = vk::ImageCreateInfo::default()
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
        Self::create_views(device, image, vkconfig, views, shader_resource_views, render_target_views)?;

        Ok((image, allocation, sampler))
    }
    /// Creates an empty image of the specified width and height
    pub fn with_dimensions(
        device: &crate::Device,
        width: u32,
        height: u32,
        depth: u32,
        layers: u32,
        vkconfig: ImageConfig,
    ) -> anyhow::Result<Self> {
        let mut basic_image_views = Vec::new();
        let mut shader_resource_views = Vec::new();
        let mut render_target_views = Vec::new();

        let (image, allocation, sampler) = Self::create_image_with_sampler_and_views(
            device, width, height, depth, layers, &vkconfig,
            &mut basic_image_views,
            &mut shader_resource_views,
            &mut render_target_views
        )?;

        Ok(Self {
            image,
            allocation: Some(allocation),
            sampler,
            basic_image_views,
            shader_resource_views,
            render_target_views,
            arbitrary_image_views: Mutex::new(Default::default()),
            config: vkconfig,
            width,
            height,
            depth,
            layers,
        })
    }
    /// Creates an image with the specified pixel data, width, height and depth
    pub fn with_data<T: Copy>(
        device: &crate::Device,
        data: &[T],
        width: u32,
        height: u32,
        depth: u32,
        vkconfig: ImageConfig,
    ) -> anyhow::Result<Self> {
        Self::with_layers(device, data, width, height, depth, 1, vkconfig)
    }
    pub fn with_layers<T: Copy>(
        device: &crate::Device,
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
            std::slice::from_raw_parts::<u8>(
                data.as_ptr() as *const u8,
                data.len() * std::mem::size_of::<T>() / std::mem::size_of::<u8>(),
            )
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
        let mut basic_image_views = Vec::new();
        let mut shader_resource_views = Vec::new();
        let mut render_target_views = Vec::new();

        let (image, allocation, sampler) = Self::create_image_with_sampler_and_views(
            device, width, height, depth, layers, &vkconfig,
            &mut basic_image_views,
            &mut shader_resource_views,
            &mut render_target_views
        )?;

        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(format_to_aspect_flags(vkconfig.format))
            .base_mip_level(0)
            .level_count(vkconfig.mip_levels)
            .layer_count(layers);

        let mut staging_buffer = crate::buffer::RawBuffer::new(
            device,
            "staging_buffer",
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

                let buffer_copy_region = [vk::BufferImageCopy::default()
                    .image_subresource(
                        vk::ImageSubresourceLayers::default()
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

                    Self::generate_mips_(
                        device.raw(),
                        cmd,
                        image,
                        width,
                        height,
                        depth,
                        layers,
                        &vkconfig,
                    );

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
                } else {
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

        staging_buffer.delete(device);
        Ok(Self {
            image,
            basic_image_views,
            shader_resource_views,
            render_target_views,
            arbitrary_image_views: Mutex::new(Default::default()),
            sampler,
            allocation: Some(allocation),
            width,
            height,
            depth,
            layers,
            config: vkconfig,
        })
    }
    /// Assumes that image is in TRANSFER_SRC_OPTIMAL layout
    pub fn generate_mips(&self, device: &crate::Device, cmd: vk::CommandBuffer) {
        Self::generate_mips_(
            device.raw(),
            cmd,
            self.image,
            self.width,
            self.height,
            self.depth,
            self.layers,
            &self.config,
        )
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

                    let image_blit = [vk::ImageBlit::default()
                        .src_subresource(
                            vk::ImageSubresourceLayers::default()
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
                            vk::ImageSubresourceLayers::default()
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

                    let mip_sub_range = vk::ImageSubresourceRange::default()
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
    pub fn shader_resource_view(&self, mip_level: usize) -> Option<vk::ImageView> {
        let ix = *self.shader_resource_views.get(mip_level)?;
        Some(self.basic_image_views[ix])
    }
    pub fn render_target_view(&self) -> Option<vk::ImageView> {
        let ix = *self.render_target_views.get(0)?;
        Some(self.basic_image_views[ix])
    }
    pub fn custom_image_view(&self, device: &crate::Device, view_desc: &ImageViewDesc) -> vk::ImageView {
        let mut arbitrary_image_views = self.arbitrary_image_views.lock();

        let view = arbitrary_image_views.entry(view_desc.clone())
        .or_insert_with(|| {
            assert!(view_desc.mip_range.clone().count() as u32 <= self.config.mip_levels);
            assert!(view_desc.layer_range.clone().count() as u32 <= self.layers);

            crate::image::view::create_view(device, self.image, &self.config, &view_desc).unwrap()
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
}