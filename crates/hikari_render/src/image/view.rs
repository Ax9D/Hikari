use ash::{vk, prelude::VkResult};
use std::{ops::Range};
use crate::{ImageConfig};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ImageViewDesc {
    pub view_type: vk::ImageViewType,
    pub aspect: vk::ImageAspectFlags,
    pub mip_range: Range<u32>,
    pub layer_range: Range<u32>,
}
pub fn create_view(device: &crate::Device, image: vk::Image, image_config: &ImageConfig, desc: &ImageViewDesc) -> VkResult<vk::ImageView>{
        let base_mip_level = desc.mip_range.start;
        let level_count = desc.mip_range.clone().count() as u32;

        let base_array_layer = desc.layer_range.start;
        let layer_count = desc.layer_range.clone().count() as u32;

        let create_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .format(image_config.format)
        .view_type(desc.view_type)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(desc.aspect)
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

        let view = unsafe { device.raw().create_image_view(&create_info, None) }?;

        Ok(view)
    }
pub fn delete_view(device: &crate::Device, view: vk::ImageView) {
    unsafe {
        device.raw().destroy_image_view(view, None);
    }
}