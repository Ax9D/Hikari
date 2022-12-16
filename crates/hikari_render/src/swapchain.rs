use std::sync::Arc;

use ash::{
    prelude::VkResult,
    vk::{self, SurfaceFormatKHR},
};

use crate::{
    renderpass::PhysicalRenderpass,
    texture::{ImageConfig, SampledImage},
};

#[derive(Clone)]
pub(crate) struct SurfaceData {
    pub surface: vk::SurfaceKHR,
    pub surface_loader: ash::extensions::khr::Surface,
}
pub struct Swapchain {
    device: Arc<crate::device::Device>,
    pub(crate) inner: vk::SwapchainKHR,
    loader: ash::extensions::khr::Swapchain,
    present_queue: Option<vk::Queue>,

    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    format: vk::Format,
    depth_image: SampledImage,

    renderpass: PhysicalRenderpass,
    framebuffers: Vec<vk::Framebuffer>,

    width: u32,
    height: u32,

    pub(crate) surface_data: SurfaceData,
}

impl Swapchain {
    pub(crate) fn create(
        device: &Arc<crate::Device>,
        width: u32,
        height: u32,
        surface_data: SurfaceData,
        old_swapchain: Option<vk::SwapchainKHR>,
        vsync: bool,
    ) -> Result<Swapchain, Box<dyn std::error::Error>> {
        let physical_device = device.physical_device();

        let swapchain_support_details = physical_device
            .get_swapchain_support_details(&surface_data.surface, &surface_data.surface_loader)?;
        let present_mode = Self::choose_present_mode(&swapchain_support_details, vsync);

        let surface_format = Self::choose_swapchain_format(&swapchain_support_details);

        let swap_extent = Self::choose_swap_extent(width, height, &swapchain_support_details);

        let mut image_count = swapchain_support_details.capabilities.min_image_count + 1;

        let max_image_count = swapchain_support_details.capabilities.max_image_count;
        if max_image_count > 0 && image_count > max_image_count {
            image_count = max_image_count;
        }

        let swapchain_loader =
            ash::extensions::khr::Swapchain::new(device.instance(), device.raw());

        let old_swapchain_vk = old_swapchain.unwrap_or(vk::SwapchainKHR::null());

        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface_data.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .image_extent(swap_extent)
            .pre_transform(swapchain_support_details.capabilities.current_transform)
            .clipped(true)
            .old_swapchain(old_swapchain_vk);

        let present_queue_ix = device
            .physical_device()
            .get_present_queue(
                device.instance(),
                &surface_data.surface,
                &surface_data.surface_loader,
            )
            .unwrap();
        let queue_family_indices = [device.unified_queue_ix, present_queue_ix];
        let present_queue;

        if queue_family_indices[0] != queue_family_indices[1] {
            present_queue = Some(unsafe { device.raw().get_device_queue(present_queue_ix, 0) });

            swapchain_create_info = swapchain_create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices)
        } else {
            present_queue = None;
            swapchain_create_info =
                swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        };

        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };

        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };

        let depth_stencil_image = SampledImage::with_dimensions(
            device,
            swap_extent.width,
            swap_extent.height,
            1,
            ImageConfig {
                format: device.supported_depth_stencil_format(),
                filtering: vk::Filter::LINEAR,
                wrap_x: vk::SamplerAddressMode::REPEAT,
                wrap_y: vk::SamplerAddressMode::REPEAT,
                wrap_z: vk::SamplerAddressMode::REPEAT,
                sampler_reduction_mode: None,
                aniso_level: 0.0,
                mip_levels: 1,
                mip_filtering: vk::SamplerMipmapMode::LINEAR,
                usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                image_type: vk::ImageType::TYPE_2D,
                image_view_type: vk::ImageViewType::TYPE_2D,
                host_readable: true,
            },
        )?;

        let image_views = Self::create_image_views(device, &images, surface_format.format)?;

        let renderpass =
            Self::create_renderpass(device, surface_format.format, &depth_stencil_image)?;

        let framebuffers = Self::create_framebuffers(
            device,
            width,
            height,
            &image_views,
            &depth_stencil_image,
            renderpass.pass,
        )?;

        log::debug!("Created swapchain");

        Ok(Swapchain {
            device: device.clone(),
            inner: swapchain,
            loader: swapchain_loader,
            present_queue,
            images,
            image_views,
            format: surface_format.format,
            width: swap_extent.width,
            height: swap_extent.height,
            depth_image: depth_stencil_image,
            renderpass,
            framebuffers,

            surface_data,
        })
    }
    fn create_renderpass(
        device: &Arc<crate::Device>,
        color_format: vk::Format,
        depth_stencil_image: &SampledImage,
    ) -> VkResult<PhysicalRenderpass> {
        let create_info = vk::RenderPassCreateInfo::builder();

        let attachments = [
            *vk::AttachmentDescription::builder()
                .format(color_format)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .samples(vk::SampleCountFlags::TYPE_1)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
            *vk::AttachmentDescription::builder()
                .format(depth_stencil_image.config().format)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .samples(vk::SampleCountFlags::TYPE_1)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL),
        ];

        let color_attachment_refs = [*vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

        let depth_stencil_attachment_ref = *vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let subpass_desc = *vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_stencil_attachment_ref);

        let subpass_descs = [subpass_desc];
        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpass_descs);

        let pass = unsafe { device.raw().create_render_pass(&create_info, None)? };

        let clear_values = vec![
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.12, 0.12, 0.12, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];
        let renderpass = PhysicalRenderpass {
            pass,
            n_color_attachments: 1,
            clear_values,
        };

        Ok(renderpass)
    }
    fn create_framebuffers(
        device: &Arc<crate::Device>,
        width: u32,
        height: u32,
        color_images: &[vk::ImageView],
        depth_stencil_image: &SampledImage,
        pass: vk::RenderPass,
    ) -> VkResult<Vec<vk::Framebuffer>> {
        let mut framebuffers = Vec::new();
        for &color_image in color_images {
            let attachments = [color_image, depth_stencil_image.image_view(1).unwrap()];

            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(pass)
                .attachments(&attachments)
                .width(width)
                .height(height)
                .layers(1);

            framebuffers.push(unsafe { device.raw().create_framebuffer(&create_info, None)? });
        }

        Ok(framebuffers)
    }
    fn choose_present_mode(
        swapchain_support_details: &crate::device::SwapchainSupportDetails,
        vsync: bool,
    ) -> vk::PresentModeKHR {
        let mailbox_supported = swapchain_support_details
            .present_modes
            .iter()
            .any(|&mode| mode == vk::PresentModeKHR::MAILBOX);

        let present_mode = if vsync {
            vk::PresentModeKHR::FIFO
        } else {
            vk::PresentModeKHR::IMMEDIATE
        };

        log::debug!("Present mode: {:?}", present_mode);

        present_mode
    }
    fn choose_swapchain_format(
        swapchain_support_details: &crate::device::SwapchainSupportDetails,
    ) -> &SurfaceFormatKHR {
        log::debug!("Supported Swapchain formats");
        swapchain_support_details
            .formats
            .iter()
            .for_each(|format| log::debug!("{:?}", format.format));

        swapchain_support_details
            .formats
            .iter()
            .find(|format| format.format == vk::Format::B8G8R8A8_UNORM)
            .expect("B8G8R8A8_UNORM surface format is not supported by device")
    }
    fn choose_swap_extent(
        width: u32,
        height: u32,
        swapchain_support_details: &crate::device::SwapchainSupportDetails,
    ) -> vk::Extent2D {
        let capabilities = &swapchain_support_details.capabilities;
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D::builder()
                .width(width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ))
                .height(height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ))
                .build()
        }
    }
    fn create_image_views(
        device: &crate::device::Device,
        images: &[vk::Image],
        format: vk::Format,
    ) -> VkResult<Vec<vk::ImageView>> {
        let mut image_views = Vec::new();
        for image in images {
            let create_info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .format(format)
                .view_type(vk::ImageViewType::TYPE_2D)
                .components(
                    vk::ComponentMapping::builder()
                        .r(vk::ComponentSwizzle::IDENTITY)
                        .g(vk::ComponentSwizzle::IDENTITY)
                        .b(vk::ComponentSwizzle::IDENTITY)
                        .a(vk::ComponentSwizzle::IDENTITY)
                        .build(),
                )
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1)
                        .build(),
                );

            let image_view = unsafe { device.raw().create_image_view(&create_info, None)? };

            image_views.push(image_view);
        }
        Ok(image_views)
    }
    pub fn depth_image(&self) -> &SampledImage {
        &self.depth_image
    }
    pub fn color_format(&self) -> vk::Format {
        self.format
    }
    pub fn depth_format(&self) -> vk::Format {
        self.depth_image.config().format
    }
    pub fn images(&self) -> &[vk::ImageView] {
        &self.image_views
    }
    pub fn renderpass(&self) -> &PhysicalRenderpass {
        &self.renderpass
    }
    pub fn framebuffers(&self) -> &[vk::Framebuffer] {
        &self.framebuffers
    }
    pub fn acquire_next_image_ix(
        &mut self,
        timeout: u64,
        signal_semaphore: vk::Semaphore,
        signal_fence: vk::Fence,
    ) -> VkResult<u32> {
        hikari_dev::profile_function!();
        unsafe {
            let (ix, _) = self.loader.acquire_next_image(
                self.inner,
                timeout,
                signal_semaphore,
                signal_fence,
            )?;
            Ok(ix)
        }
    }
    pub fn present(&mut self, image_ix: u32, wait_semaphone: vk::Semaphore) -> VkResult<bool> {
        let swapchains = [self.inner];
        let wait_semaphones = [wait_semaphone];
        let image_ixs = [image_ix];
        let present_info = vk::PresentInfoKHR::builder()
            .swapchains(&swapchains)
            .wait_semaphores(&wait_semaphones)
            .image_indices(&image_ixs);

        unsafe {
            if let Some(present_queue) = self.present_queue {
                self.loader.queue_present(present_queue, &present_info)
            } else {
                self.loader
                    .queue_present(*self.device.unified_queue(), &present_info)
            }
        }
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn size(&self) -> (u32, u32) {
        (self.width(), self.height())
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        for &framebuffer in &self.framebuffers {
            unsafe {
                self.device.raw().destroy_framebuffer(framebuffer, None);
            }
        }

        unsafe {
            self.device
                .raw()
                .destroy_render_pass(self.renderpass.pass, None);
        };

        for image_view in self.image_views.drain(..) {
            unsafe {
                self.device.raw().destroy_image_view(image_view, None);
            }
        }

        unsafe {
            self.loader.destroy_swapchain(self.inner, None);
        }
        log::debug!("Dropped swapchain");
    }
}
