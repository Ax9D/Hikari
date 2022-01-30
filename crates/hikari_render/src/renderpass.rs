use ash::vk;

#[derive(Clone)]
pub struct PhysicalRenderpass {
    pub pass: vk::RenderPass,
    pub n_color_attachments: usize,
    pub clear_values: Vec<vk::ClearValue>,
}
