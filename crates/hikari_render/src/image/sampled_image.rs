use std::sync::Arc;

use ash::vk;

use crate::{format_size, ImageConfig, RawSampledImage, ImageViewDesc, format_to_aspect_flags, Buffer, bindless::BindlessHandle};


/// An Image that can be sampled in shaders
/// An ImageView is generated for each mip level automatically
pub struct SampledImage {
    device: Arc<crate::Device>,
    raw: RawSampledImage,
    download_buffer: Option<crate::buffer::CpuBuffer<u8>>,
    bindless_handles: Vec<BindlessHandle<vk::ImageView>>
}

impl SampledImage {
    /// Creates an empty image of the specified width and height
    pub fn with_dimensions(
        device: &Arc<crate::Device>,
        width: u32,
        height: u32,
        depth: u32,
        layers: u32,
        vkconfig: ImageConfig,
    ) -> anyhow::Result<Self> {
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

        let raw = RawSampledImage::with_dimensions(device, width, height, depth, layers, vkconfig)?;

        let bindless_handles = Self::create_bindless(device, &raw);

        Ok(Self {
            device: device.clone(),
            raw,
            download_buffer,
            bindless_handles,
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
        vkconfig: ImageConfig,
    ) -> anyhow::Result<Self> {
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

        let raw = RawSampledImage::with_layers(device, data, width, height, depth, layers, vkconfig)?;


        let bindless_handles = Self::create_bindless(device, &raw);
        Ok(Self {
            device: device.clone(),
            raw,
            download_buffer,
            bindless_handles,
        })
    }
    fn create_bindless(device: &Arc<crate::Device>, raw: &RawSampledImage) -> Vec<BindlessHandle<vk::ImageView>> {
        let mut handles = Vec::new();
        for &view_ix in &raw.shader_resource_views {
            let shader_resource_view = raw.basic_image_views[view_ix];
            let sampler = raw.sampler;
            let usage = raw.config.usage;
            let handle = device.bindless_resources().allocate_image(device, shader_resource_view, sampler, usage);
            handles.push(handle);
        } 
        handles
    }
    pub fn bindless_handle(&self, mip_level: u32) -> BindlessHandle<vk::ImageView> {
        let ix = self.raw.shader_resource_views[mip_level as usize];
        self.bindless_handles[ix]
    }
    /// Assumes that image is in TRANSFER_SRC_OPTIMAL layout
    #[inline]
    pub fn generate_mips(&self, cmd: vk::CommandBuffer) {
        self.raw.generate_mips(&self.device, cmd)
    }
    #[inline]
    pub fn image(&self) -> vk::Image {
        self.raw.image()
    }
    #[inline]
    pub fn shader_resource_view(&self, mip_level: u32) -> Option<vk::ImageView> {
        self.raw.shader_resource_view(mip_level as usize)
    }
    #[inline]
    pub fn render_target_view(&self) -> Option<vk::ImageView> {
        self.raw.render_target_view()
    }
    #[inline]
    pub fn custom_image_view(&self, view_desc: &ImageViewDesc) -> vk::ImageView {
        self.raw.custom_image_view(&self.device, view_desc)
    }
    #[inline]
    pub fn sampler(&self) -> vk::Sampler {
        self.raw.sampler()
    }
    #[inline]
    pub fn width(&self) -> u32 {
        self.raw.width()
    }
    #[inline]
    pub fn height(&self) -> u32 {
        self.raw.height()
    }
    #[inline]
    pub fn depth(&self) -> u32 {
        self.raw.depth()
    }
    #[inline]
    pub fn layers(&self) -> u32 {
        self.raw.layers()
    }
    #[inline]
    pub fn config(&self) -> &ImageConfig {
        self.raw.config()
    }

    /// Copies the image from the GPU to the Host; the read is not synchronized on the GPU, the caller must ensure the image is not being used on the GPU
    /// Returns a slice to the downloaded data if the image was created with `host_readable` set to `true` in the `ImageConfig`
    /// Otherwise returns `None`
    pub fn download(&self, cmd: vk::CommandBuffer, mip_level: u32, layout: vk::ImageLayout, trigger_event: vk::Event) -> Option<&[u8]> {
        if let Some(ref download_buffer) = self.download_buffer {
            assert!(mip_level > 0);

            unsafe {
            let device = self.device.raw();

            let subresource_range = vk::ImageSubresourceRange::default()
                .layer_count(1)
                .level_count(mip_level)
                .aspect_mask(format_to_aspect_flags(self.raw.config().format));

            crate::barrier::image_memory_barrier(
                device,
                cmd,
                self.image(),
                subresource_range,
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_READ,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
            );

            let regions = [vk::BufferImageCopy::default()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(
                    vk::ImageSubresourceLayers::default()
                        .mip_level(0)
                        .aspect_mask(format_to_aspect_flags(self.config().format))
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D {
                    width: self.width(),
                    height: self.height(),
                    depth: self.depth(),
                })];

            device.cmd_copy_image_to_buffer(
                cmd,
                self.image(),
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                download_buffer.buffer(),
                &regions,
            );

            crate::barrier::image_memory_barrier(
                device,
                cmd,
                self.image(),
                subresource_range,
                vk::AccessFlags::TRANSFER_READ,
                vk::AccessFlags::empty(),
                vk::ImageLayout::UNDEFINED,
                layout,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
            );

            let dep_info = vk::DependencyInfo::default();
            device.cmd_set_event2(cmd, trigger_event, &dep_info);
            }
            let slice = download_buffer.mapped_slice();
            return Some(slice);
        }

        None
    }
}


impl Drop for SampledImage {
    #[inline]
    fn drop(&mut self) {
        //self.raw.delete(&self.device);
        use crate::delete::DeleteRequest;
        let deleter = self.device.deleter();

        for image_view in self.raw.basic_image_views.drain(..) {
            deleter.request_delete(DeleteRequest::ImageView(image_view));
        }
        
        for (_, image_view) in self.raw.arbitrary_image_views.lock().drain() {
            deleter.request_delete(DeleteRequest::ImageView(image_view));
        }

        let allocation = self.raw.allocation.take().unwrap();
        deleter.request_delete(DeleteRequest::Image(self.raw.image, allocation));

        for handle in self.bindless_handles.drain(..) {
            let request = DeleteRequest::BindlessImage(handle);
            deleter.request_delete(request);
        }
    }
}
