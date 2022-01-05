use std::sync::Arc;

use ash::{prelude::VkResult, vk};

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
        vk::Format::D16_UNORM => 2,
        vk::Format::D16_UNORM_S8_UINT => 3,
        vk::Format::D24_UNORM_S8_UINT => 3,
        vk::Format::D32_SFLOAT => 4,
        vk::Format::D32_SFLOAT_S8_UINT => 5,
        _ => todo!(),
    }
}
pub struct SampledImage {
    device: Arc<crate::Device>,
    allocation: gpu_allocator::vulkan::Allocation,
    image: vk::Image,
    image_views: Vec<vk::ImageView>,
    sampler: vk::Sampler,
    config: ImageConfig,
    width: u32,
    height: u32,

    download_buffer: Option<crate::buffer::CpuBuffer<u8>>,
}

#[derive(Copy, Clone, Debug)]
pub struct ImageConfig {
    pub format: vk::Format,
    pub filtering: vk::Filter,
    pub wrap_x: vk::SamplerAddressMode,
    pub wrap_y: vk::SamplerAddressMode,
    pub aniso_level: u8,
    pub mip_levels: u32,
    pub mip_filtering: vk::SamplerMipmapMode,
    pub usage: vk::ImageUsageFlags,
    pub image_type: vk::ImageType,
    pub host_readable: bool,
}

impl ImageConfig {
    pub fn color2d() -> Self {
        Self {
            format: vk::Format::R8G8B8A8_UNORM,
            filtering: vk::Filter::LINEAR,
            wrap_x: vk::SamplerAddressMode::REPEAT,
            wrap_y: vk::SamplerAddressMode::REPEAT,
            aniso_level: 0,
            mip_levels: 1,
            mip_filtering: vk::SamplerMipmapMode::LINEAR,
            usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            image_type: vk::ImageType::TYPE_2D,
            host_readable: false,
        }
    }
    pub fn depth_stencil() -> Self {
        Self {
            format: vk::Format::D24_UNORM_S8_UINT,
            filtering: vk::Filter::LINEAR,
            wrap_x: vk::SamplerAddressMode::REPEAT,
            wrap_y: vk::SamplerAddressMode::REPEAT,
            aniso_level: 0,
            mip_levels: 1,
            mip_filtering: vk::SamplerMipmapMode::LINEAR,
            usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            image_type: vk::ImageType::TYPE_2D,
            host_readable: false,
        }
    }
}

pub(crate) fn usage_to_aspect_flags(usage: vk::ImageUsageFlags) -> vk::ImageAspectFlags {
    use vk::ImageAspectFlags as af;
    use vk::ImageUsageFlags as us;

    if usage.contains(us::COLOR_ATTACHMENT) {
        af::COLOR
    } else if usage.contains(us::DEPTH_STENCIL_ATTACHMENT) {
        af::DEPTH | af::STENCIL
    } else if usage.contains(us::SAMPLED) {
        af::COLOR
    } else {
        panic!("Unsupported usage")
    }
}

impl SampledImage {
    fn create_sampler(
        device: &Arc<crate::Device>,
        vkconfig: &ImageConfig,
    ) -> VkResult<vk::Sampler> {
        let mut create_info = *vk::SamplerCreateInfo::builder()
            .min_filter(vkconfig.filtering)
            .mag_filter(vkconfig.filtering)
            .mipmap_mode(vkconfig.mip_filtering)
            .compare_op(vk::CompareOp::NEVER)
            .min_lod(0.0)
            .max_lod(0.0)
            .border_color(vk::BorderColor::INT_OPAQUE_WHITE)
            .anisotropy_enable(false)
            .address_mode_u(vkconfig.wrap_x)
            .address_mode_v(vkconfig.wrap_y)
            .max_anisotropy(1.0);

        if vkconfig.mip_levels > 1 {
            create_info.max_lod = vkconfig.mip_levels as f32;
        }

        if vkconfig.aniso_level > 0
            && device.is_feature_supported(crate::device::Features::SAMPLER_ANISOTROPY)
        {
            create_info.max_anisotropy = vkconfig.aniso_level as f32;
            create_info.anisotropy_enable = vk::TRUE;
        }

        unsafe { device.raw().create_sampler(&create_info, None) }
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
                .view_type(vk::ImageViewType::TYPE_2D)
                .subresource_range(
                    *vk::ImageSubresourceRange::builder()
                        .aspect_mask(usage_to_aspect_flags(vkconfig.usage))
                        .base_mip_level(mip_level)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                });
            views.push(unsafe { device.raw().create_image_view(&create_info, None)? });
        }

        Ok(views)
    }
    fn create_image_with_sampler_and_views(
        device: &Arc<crate::Device>,
        width: u32,
        height: u32,
        vkconfig: &ImageConfig,
    ) -> Result<
        (
            vk::Image,
            gpu_allocator::vulkan::Allocation,
            vk::Sampler,
            Vec<vk::ImageView>,
        ),
        Box<dyn std::error::Error>,
    > {
        log::info!("format {:?}", vkconfig.usage);

        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vkconfig.format)
            .mip_levels(vkconfig.mip_levels)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .sharing_mode(vk::SharingMode::EXCLUSIVE /**/)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .usage(
                vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::TRANSFER_SRC
                    | vkconfig.usage,
            );

        let (image, allocation) = crate::texture::create_image(
            device,
            &image_create_info,
            gpu_allocator::MemoryLocation::GpuOnly,
        )?;

        let sampler = Self::create_sampler(device, vkconfig)?;
        let image_views = Self::create_views(device, image, vkconfig)?;

        Ok((image, allocation, sampler, image_views))
    }
    pub fn with_dimensions(
        device: &Arc<crate::Device>,
        width: u32,
        height: u32,
        vkconfig: ImageConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (image, allocation, sampler, image_views) =
            Self::create_image_with_sampler_and_views(device, width, height, &vkconfig)?;

        let download_buffer = if vkconfig.host_readable {
            Some(crate::buffer::CpuBuffer::new(
                device,
                (width * height * format_size(vkconfig.format)) as usize,
                vk::BufferUsageFlags::TRANSFER_DST,
                gpu_allocator::MemoryLocation::GpuToCpu,
            )?)
        } else {
            None
        };

        Ok(Self {
            device: device.clone(),
            image,
            allocation,
            sampler,
            image_views,
            config: vkconfig,
            width,
            height,
            download_buffer,
        })
    }
    pub fn with_data(
        device: &Arc<crate::Device>,
        data: &[u8],
        width: u32,
        height: u32,
        mut vkconfig: ImageConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        //log::warn!("Mip levels: {}", mip_levels);

        // let image_create_info = vk::ImageCreateInfo::builder()
        //     .image_type(vk::ImageType::TYPE_2D)
        //     .format(vkconfig.format)
        //     .mip_levels(vkconfig.mip_levels)
        //     .array_layers(1)
        //     .samples(vk::SampleCountFlags::TYPE_1)
        //     .tiling(vk::ImageTiling::OPTIMAL)
        //     .sharing_mode(vk::SharingMode::EXCLUSIVE /**/)
        //     .initial_layout(vk::ImageLayout::UNDEFINED)
        //     .extent(vk::Extent3D {
        //         width,
        //         height,
        //         depth: 1,
        //     })
        //     .usage(
        //         vk::ImageUsageFlags::TRANSFER_DST
        //             | vk::ImageUsageFlags::TRANSFER_SRC
        //             | vk::ImageUsageFlags::SAMPLED,
        //     );

        // let (image, allocation) = crate::texture::create_image(
        //     device,
        //     &image_create_info,
        //     gpu_allocator::MemoryLocation::GpuOnly,
        // )?;

        // let sampler = Self::create_sampler(device, &vkconfig)?;
        // let image_views = Self::create_views(device, image, &vkconfig)?;

        vkconfig.usage |= vk::ImageUsageFlags::TRANSFER_DST;

        if vkconfig.host_readable {
            vkconfig.usage |= vk::ImageUsageFlags::TRANSFER_SRC;
        }

        let image_buffer_max_size =
            (width * height) as usize * format_size(vkconfig.format) as usize;

        //FIX ME: This is probably wrong?, Dont assume format sizes
        if data.len() != image_buffer_max_size {
            return Err(format!(
                "Cannot create gpu image, data size {} bytes doesn't match expected size {} bytes, format is {:?}",
                data.len(),
                image_buffer_max_size,
                vkconfig.format
            )
            .into());
        }
        let (image, allocation, sampler, image_views) =
            Self::create_image_with_sampler_and_views(device, width, height, &vkconfig)?;

        let subresource_range = *vk::ImageSubresourceRange::builder()
            .aspect_mask(usage_to_aspect_flags(vkconfig.usage))
            .level_count(1)
            .layer_count(1);

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
                            .aspect_mask(usage_to_aspect_flags(vkconfig.usage))
                            .mip_level(0)
                            .base_array_layer(0)
                            .layer_count(1),
                    )
                    .image_extent(vk::Extent3D {
                        width,
                        height,
                        depth: 1,
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
                    Self::generate_mips(device.raw(), cmd, image, width, height, &vkconfig);
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
            image_views,
            sampler,
            allocation,
            width,
            height,
            config: vkconfig,
            download_buffer,
        })
    }
    fn generate_mips(
        device: &ash::Device,
        cmd: vk::CommandBuffer,
        image: vk::Image,
        width: u32,
        height: u32,
        config: &ImageConfig,
    ) {
        let levels = config.mip_levels;

        let subresource_range = *vk::ImageSubresourceRange::builder()
            .aspect_mask(usage_to_aspect_flags(config.usage))
            .level_count(1)
            .layer_count(1);

        crate::barrier::image_memory_barrier(
            device,
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

        unsafe {
            let mut mip_width = width as i32;
            let mut mip_height = height as i32;

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

                let image_blit = [*vk::ImageBlit::builder()
                    .src_subresource(
                        *vk::ImageSubresourceLayers::builder()
                            .aspect_mask(usage_to_aspect_flags(config.usage))
                            .layer_count(1)
                            .mip_level(level - 1)
                            .base_array_layer(0),
                    )
                    .src_offsets([
                        vk::Offset3D { x: 0, y: 0, z: 0 },
                        vk::Offset3D {
                            x: mip_width,
                            y: mip_height,
                            z: 1,
                        },
                    ])
                    .dst_subresource(
                        *vk::ImageSubresourceLayers::builder()
                            .aspect_mask(usage_to_aspect_flags(config.usage))
                            .layer_count(1)
                            .mip_level(level)
                            .base_array_layer(0),
                    )
                    .dst_offsets([
                        vk::Offset3D { x: 0, y: 0, z: 0 },
                        vk::Offset3D {
                            x: next_mip_width,
                            y: next_mip_height,
                            z: 1,
                        },
                    ])];

                let mip_sub_range = *vk::ImageSubresourceRange::builder()
                    .aspect_mask(usage_to_aspect_flags(config.usage))
                    .base_mip_level(level)
                    .level_count(1)
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
            }

            crate::barrier::image_memory_barrier(
                device,
                cmd,
                image,
                subresource_range,
                vk::AccessFlags::TRANSFER_READ,
                vk::AccessFlags::SHADER_READ,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            );
        }

        log::debug!("Generated {} mips", levels);
    }
    pub fn image(&self) -> vk::Image {
        self.image
    }
    pub fn image_view(&self, mip_level: usize) -> Option<vk::ImageView> {
        self.image_views.get(mip_level - 1).copied()
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
    pub fn config(&self) -> &ImageConfig {
        &self.config
    }

    /// Copies the image from the GPU to the Host; the read is not synchronized on the GPU, the caller must ensure the image is not being used on the GPU
    pub fn download(&self, mip_level: u32) -> Option<&[u8]> {
        if let Some(ref download_buffer) = self.download_buffer {
            assert!(mip_level > 0);
            let now = std::time::Instant::now();

            unsafe {
                self.device
                    .submit_commands_immediate(|cmd| {
                        let device = self.device.raw();

                        let subresource_range = *vk::ImageSubresourceRange::builder()
                            .layer_count(1)
                            .level_count(mip_level)
                            .aspect_mask(usage_to_aspect_flags(self.config.usage));

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
                                    .aspect_mask(usage_to_aspect_flags(self.config.usage))
                                    .base_array_layer(0)
                                    .layer_count(1),
                            )
                            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                            .image_extent(vk::Extent3D {
                                width: self.width,
                                height: self.height,
                                depth: 1,
                            })];

                        device.cmd_copy_image_to_buffer(
                            cmd,
                            self.image,
                            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                            download_buffer.buffer(),
                            &regions,
                        );

                        Ok(())
                    })
                    .ok()?
            }
            println!("{:?}", now.elapsed());
            let slice = download_buffer.mapped_slice();
            return Some(slice);
        }

        None
    }
}

impl Drop for SampledImage {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_sampler(self.sampler, None);

            for &image_view in &self.image_views {
                self.device.raw().destroy_image_view(image_view, None);
            }

            crate::texture::delete_image(&self.device, self.image, self.allocation.clone())
                .unwrap();
        }
    }
}
