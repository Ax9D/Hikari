use ash::vk;

#[derive(Clone)]
pub struct PhysicalRenderpass {
    pub pass: vk::RenderPass,
    pub n_color_attachments: usize,
    pub clear_values: Vec<vk::ClearValue>,
}

pub fn delete(device: &crate::Device, renderpass: vk::RenderPass) {
    unsafe { device.raw().destroy_render_pass(renderpass, None) }
}