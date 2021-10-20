use ash::vk;

pub fn image_memory_barrier(
    device: &ash::Device,
    cmd: vk::CommandBuffer,
    image: vk::Image,
    subresource_range: vk::ImageSubresourceRange,
    src_access: vk::AccessFlags,
    dst_access: vk::AccessFlags,
    initial_layout: vk::ImageLayout,
    final_layout: vk::ImageLayout,
    src_stage: vk::PipelineStageFlags,
    dst_stage: vk::PipelineStageFlags,
) {
    let barrier = [*vk::ImageMemoryBarrier::builder()
        .src_access_mask(src_access)
        .dst_access_mask(dst_access)
        .old_layout(initial_layout)
        .new_layout(final_layout)
        .image(image)
        .subresource_range(subresource_range)];
    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &barrier,
        );
    }
}
