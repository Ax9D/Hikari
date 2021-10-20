use std::sync::Arc;

use ash::{prelude::VkResult, vk};

use super::AllocationData;

// pub(crate) struct Framebuffer {
//     device: Arc<crate::Device>,
//     inner: vk::Framebuffer,
//     attachment_ixs: Vec<usize>,
//     renderpass: vk::RenderPass,
// }

pub(super) fn from_allocation_data(
    device: &Arc<crate::Device>,
    allocation_data: &AllocationData,
    attachment_ixs: &[usize],
    renderpass: vk::RenderPass,
) -> VkResult<vk::Framebuffer> {
    let images: Vec<_> = attachment_ixs
        .iter()
        .map(|&ix| allocation_data.get_image(ix).unwrap())
        .collect();

    let image_views: Vec<_> = images
        .iter()
        .map(|image| image.image_view(1).unwrap())
        .collect();

    let width = images
        .iter()
        .max_by(|a, b| a.width().cmp(&b.width()))
        .unwrap()
        .width();
    let height = images
        .iter()
        .max_by(|a, b| a.height().cmp(&b.height()))
        .unwrap()
        .height();

    let create_info = vk::FramebufferCreateInfo::builder()
        .render_pass(renderpass)
        .attachments(&image_views)
        .width(width)
        .height(height)
        .layers(1);
    Ok(unsafe { device.raw().create_framebuffer(&create_info, None)? })
}
// pub(super) fn from_swapchain_images(allocation_data: &AllocationData, swapchain_color: &[vk::ImageView], swapchain_depth: Option<vk::ImageView>, allocated_image_ixs: &[usize], image_ordering: &[Option<usize>]) {

// }
