use ash::{prelude::VkResult, vk};

use crate::image::SampledImage;

// pub(crate) struct Framebuffer {
//     device: Arc<crate::Device>,
//     inner: vk::Framebuffer,
//     attachment_ixs: Vec<usize>,
//     renderpass: vk::RenderPass,
// }

pub(super) fn from_allocation_data(
    device: &crate::Device,
    images: &[&SampledImage],
    renderpass: vk::RenderPass,
) -> VkResult<vk::Framebuffer> {
    let image_views: Vec<_> = images
        .iter()
        .map(|image| image.render_target_view().unwrap())
        .collect();

    let width = images
        .iter()
        .max_by(|a, b| a.width().cmp(&b.width()))
        .map(|image| image.width())
        .unwrap_or(0);

    let height = images
        .iter()
        .max_by(|a, b| a.height().cmp(&b.height()))
        .map(|image| image.height())
        .unwrap_or(0);

    let create_info = vk::FramebufferCreateInfo::builder()
        .render_pass(renderpass)
        .attachments(&image_views)
        .width(width)
        .height(height)
        .layers(1);
    Ok(unsafe { device.raw().create_framebuffer(&create_info, None)? })
}
pub(crate) fn delete(device: &crate::Device, framebuffer: vk::Framebuffer) {
    unsafe { device.raw().destroy_framebuffer(framebuffer, None) }
}
// pub(super) fn from_swapchain_images(allocation_data: &AllocationData, swapchain_color: &[vk::ImageView], swapchain_depth: Option<vk::ImageView>, allocated_image_ixs: &[usize], image_ordering: &[Option<usize>]) {

// }
