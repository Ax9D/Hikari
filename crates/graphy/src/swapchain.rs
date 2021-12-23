use std::sync::Arc;

use ash::{
    extensions::khr::Surface,
    prelude::VkResult,
    vk::{self, SurfaceFormatKHR},
};
use winit::window::Window;

use crate::texture::{SampledImage, VkTextureConfig};

pub(crate) struct Swapchain {
    device: Arc<crate::device::Device>,
    inner: vk::SwapchainKHR,
    loader: ash::extensions::khr::Swapchain,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    format: vk::Format,
    depth_image: SampledImage,

    renderpass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,

    width: u32,
    height: u32,
}

impl Swapchain {
    pub fn create(
        device: &Arc<crate::Device>,
        window: &Window,
        surface: &vk::SurfaceKHR,
        surface_loader: Surface,
    ) -> Result<Swapchain, Box<dyn std::error::Error>> {
        let physical_device = device.physical_device();
        //let swapchain_support_details = physical_device.swapchain_support_details();

        let present_mode = Self::choose_present_mode(physical_device.swapchain_support_details());

        let surface_format =
            Self::choose_swapchain_format(physical_device.swapchain_support_details())?;

        let swap_extent =
            Self::choose_swap_extent(window, physical_device.swapchain_support_details());

        let mut image_count = physical_device
            .swapchain_support_details()
            .capabilities
            .min_image_count
            + 1;

        let max_image_count = physical_device
            .swapchain_support_details()
            .capabilities
            .max_image_count;
        if max_image_count > 0 && image_count > max_image_count {
            image_count = max_image_count;
        }

        let swapchain_loader =
            ash::extensions::khr::Swapchain::new(device.instance(), device.raw());

        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .image_extent(swap_extent)
            .pre_transform(
                physical_device
                    .swapchain_support_details()
                    .capabilities
                    .current_transform,
            )
            .clipped(true);

        let queue_family_indices = [
            physical_device.graphics_queue_index(),
            physical_device.present_queue_index(),
        ];

        swapchain_create_info = if queue_family_indices[0] != queue_family_indices[1] {
            swapchain_create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices)
        } else {
            swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        };

        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };

        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };

        let depth_stencil_image = SampledImage::with_dimensions(
            device,
            swap_extent.width,
            swap_extent.height,
            VkTextureConfig {
                format: vk::Format::D24_UNORM_S8_UINT,
                filtering: vk::Filter::LINEAR,
                wrap_x: vk::SamplerAddressMode::REPEAT,
                wrap_y: vk::SamplerAddressMode::REPEAT,
                aniso_level: 0,
                mip_levels: 1,
                mip_filtering: vk::SamplerMipmapMode::LINEAR,
                aspect_flags: vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
                primary_image_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                host_readable: true,
                usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            },
        )?;

        let image_views = Self::create_image_views(device, &images, surface_format.format)?;

        let (renderpass, framebuffers) = Self::create_render_pass_and_framebuffer(
            device,
            swap_extent.width,
            swap_extent.height,
            &image_views,
            surface_format.format,
            &depth_stencil_image,
        )?;

        log::debug!("Created swapchain");

        Ok(Swapchain {
            device: device.clone(),
            inner: swapchain,
            loader: swapchain_loader,
            images,
            image_views,
            format: surface_format.format,
            width: swap_extent.width,
            height: swap_extent.height,
            depth_image: depth_stencil_image,
            renderpass,
            framebuffers,
        })
    }
    fn create_render_pass_and_framebuffer(
        device: &Arc<crate::Device>,
        width: u32,
        height: u32,
        color_images: &Vec<vk::ImageView>,
        color_format: vk::Format,
        depth_stencil_image: &SampledImage,
    ) -> VkResult<(vk::RenderPass, Vec<vk::Framebuffer>)> {
        let create_info = vk::RenderPassCreateInfo::builder();

        let attachments = [
            *vk::AttachmentDescription::builder()
                .format(color_format)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .samples(vk::SampleCountFlags::TYPE_1)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
            *vk::AttachmentDescription::builder()
                .format(depth_stencil_image.config().format)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
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

        let renderpass = unsafe { device.raw().create_render_pass(&create_info, None)? };

        let mut framebuffers = Vec::new();
        for &color_image in color_images {
            let attachments = [color_image, depth_stencil_image.image_view(1).unwrap()];

            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass)
                .attachments(&attachments)
                .width(width)
                .height(height)
                .layers(1);

            framebuffers.push(unsafe { device.raw().create_framebuffer(&create_info, None)? });
        }

        Ok((renderpass, framebuffers))
    }
    fn choose_present_mode(
        swapchain_support_details: &crate::device::SwapchainSupportDetails,
    ) -> vk::PresentModeKHR {
        let mailbox_supported = swapchain_support_details
            .present_modes
            .iter()
            .find(|&&mode| mode == vk::PresentModeKHR::MAILBOX)
            .is_some();

        if mailbox_supported {
            vk::PresentModeKHR::MAILBOX
        } else {
            vk::PresentModeKHR::FIFO
        }
    }
    fn choose_swapchain_format(
        swapchain_support_details: &crate::device::SwapchainSupportDetails,
    ) -> Result<&SurfaceFormatKHR, String> {
        log::debug!("Supported Swapchain formats");
        swapchain_support_details
            .formats
            .iter()
            .for_each(|format| log::debug!("{:?}", format.format));

        swapchain_support_details
            .formats
            .iter()
            .find(|format| format.format == vk::Format::B8G8R8A8_UNORM)
            .ok_or("B8G8R8A8_UNORM surface format is not supported by device".into())
    }
    fn choose_swap_extent(
        window: &Window,
        swapchain_support_details: &crate::device::SwapchainSupportDetails,
    ) -> vk::Extent2D {
        let capabilities = &swapchain_support_details.capabilities;
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            let physical_size = window.inner_size();
            vk::Extent2D::builder()
                .width(physical_size.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ))
                .height(physical_size.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ))
                .build()
        }
    }
    fn create_image_views(
        device: &crate::device::Device,
        images: &Vec<vk::Image>,
        format: vk::Format,
    ) -> Result<Vec<vk::ImageView>, Box<dyn std::error::Error>> {
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
    pub fn renderpass(&self) -> vk::RenderPass {
        self.renderpass
    }
    pub fn framebuffers(&self) -> &[vk::Framebuffer] {
        &self.framebuffers
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
            self.device.raw().destroy_render_pass(self.renderpass, None);
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
