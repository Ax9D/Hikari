use ash::vk;
use vk_sync_fork::AccessType;

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

pub fn to_sync2_access_flags(flags: vk::AccessFlags) -> vk::AccessFlags2KHR {
    vk::AccessFlags2KHR::from_raw(flags.as_raw() as u64)
    // match flags {
    //     vk::AccessFlags::INDIRECT_COMMAND_READ => vk::AccessFlags2KHR::INDIRECT_COMMAND_READ,
    //     vk::AccessFlags::INDEX_READ => vk::AccessFlags2KHR::INDEX_READ,
    //     vk::AccessFlags::VERTEX_ATTRIBUTE_READ => vk::AccessFlags2KHR::VERTEX_ATTRIBUTE_READ,
    //     vk::AccessFlags::UNIFORM_READ => vk::AccessFlags2KHR::UNIFORM_READ,
    //     vk::AccessFlags::INPUT_ATTACHMENT_READ => vk::AccessFlags2KHR::INPUT_ATTACHMENT_READ,
    //     vk::AccessFlags::SHADER_READ => vk::AccessFlags2KHR::SHADER_READ,
    //     vk::AccessFlags::SHADER_WRITE => vk::AccessFlags2KHR::SHADER_WRITE,
    //     vk::AccessFlags::COLOR_ATTACHMENT_READ => vk::AccessFlags2KHR::COLOR_ATTACHMENT_READ,
    //     vk::AccessFlags::COLOR_ATTACHMENT_WRITE => vk::AccessFlags2KHR::COLOR_ATTACHMENT_WRITE,
    //     vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ => {
    //         vk::AccessFlags2KHR::DEPTH_STENCIL_ATTACHMENT_READ
    //     }
    //     vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE => {
    //         vk::AccessFlags2KHR::DEPTH_STENCIL_ATTACHMENT_WRITE
    //     }
    //     vk::AccessFlags::TRANSFER_READ => vk::AccessFlags2KHR::TRANSFER_READ,
    //     vk::AccessFlags::TRANSFER_WRITE => vk::AccessFlags2KHR::TRANSFER_WRITE,
    //     vk::AccessFlags::HOST_READ => vk::AccessFlags2KHR::HOST_READ,
    //     vk::AccessFlags::HOST_WRITE => vk::AccessFlags2KHR::HOST_WRITE,
    //     vk::AccessFlags::MEMORY_READ => vk::AccessFlags2KHR::MEMORY_READ,
    //     vk::AccessFlags::MEMORY_WRITE => vk::AccessFlags2KHR::MEMORY_WRITE,
    //     _ => unreachable!(),
    // }
}
pub fn to_sync2_stage_flags(flags: vk::PipelineStageFlags) -> vk::PipelineStageFlags2KHR {
    vk::PipelineStageFlags2KHR::from_raw(flags.as_raw() as u64)
    // match flags {
    //     vk::PipelineStageFlags::TOP_OF_PIPE => vk::PipelineStageFlags2KHR::TOP_OF_PIPE,
    //     vk::PipelineStageFlags::DRAW_INDIRECT => vk::PipelineStageFlags2KHR::DRAW_INDIRECT,
    //     vk::PipelineStageFlags::VERTEX_INPUT => vk::PipelineStageFlags2KHR::VERTEX_INPUT,
    //     vk::PipelineStageFlags::VERTEX_SHADER => vk::PipelineStageFlags2KHR::VERTEX_SHADER,
    //     vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER => {
    //         vk::PipelineStageFlags2KHR::TESSELLATION_CONTROL_SHADER
    //     }
    //     vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER => {
    //         vk::PipelineStageFlags2KHR::TESSELLATION_EVALUATION_SHADER
    //     }
    //     vk::PipelineStageFlags::GEOMETRY_SHADER => vk::PipelineStageFlags2KHR::GEOMETRY_SHADER,
    //     vk::PipelineStageFlags::FRAGMENT_SHADER => vk::PipelineStageFlags2KHR::FRAGMENT_SHADER,
    //     vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS => {
    //         vk::PipelineStageFlags2KHR::EARLY_FRAGMENT_TESTS
    //     }
    //     vk::PipelineStageFlags::LATE_FRAGMENT_TESTS => {
    //         vk::PipelineStageFlags2KHR::LATE_FRAGMENT_TESTS
    //     }
    //     vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT => {
    //         vk::PipelineStageFlags2KHR::COLOR_ATTACHMENT_OUTPUT
    //     }
    //     vk::PipelineStageFlags::COMPUTE_SHADER => vk::PipelineStageFlags2KHR::COMPUTE_SHADER,
    //     vk::PipelineStageFlags::TRANSFER => vk::PipelineStageFlags2KHR::TRANSFER,
    //     vk::PipelineStageFlags::BOTTOM_OF_PIPE => vk::PipelineStageFlags2KHR::BOTTOM_OF_PIPE,
    //     vk::PipelineStageFlags::HOST => vk::PipelineStageFlags2KHR::HOST,
    //     vk::PipelineStageFlags::ALL_GRAPHICS => vk::PipelineStageFlags2KHR::ALL_GRAPHICS,
    //     vk::PipelineStageFlags::ALL_COMMANDS => vk::PipelineStageFlags2KHR::ALL_COMMANDS,

    //     _ => unreachable!(),
    // }
}
fn is_read(access: &AccessType) -> bool {
    match access {
        AccessType::Nothing
        | AccessType::CommandBufferReadNVX
        | AccessType::IndirectBuffer
        | AccessType::IndexBuffer
        | AccessType::VertexBuffer
        | AccessType::VertexShaderReadUniformBuffer
        | AccessType::VertexShaderReadSampledImageOrUniformTexelBuffer
        | AccessType::VertexShaderReadOther
        | AccessType::TessellationControlShaderReadUniformBuffer
        | AccessType::TessellationControlShaderReadSampledImageOrUniformTexelBuffer
        | AccessType::TessellationControlShaderReadOther
        | AccessType::TessellationEvaluationShaderReadUniformBuffer
        | AccessType::TessellationEvaluationShaderReadSampledImageOrUniformTexelBuffer
        | AccessType::TessellationEvaluationShaderReadOther
        | AccessType::GeometryShaderReadUniformBuffer
        | AccessType::GeometryShaderReadSampledImageOrUniformTexelBuffer
        | AccessType::GeometryShaderReadOther
        | AccessType::FragmentShaderReadUniformBuffer
        | AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer
        | AccessType::FragmentShaderReadColorInputAttachment
        | AccessType::FragmentShaderReadDepthStencilInputAttachment
        | AccessType::FragmentShaderReadOther
        | AccessType::ColorAttachmentRead
        | AccessType::DepthStencilAttachmentRead
        | AccessType::ComputeShaderReadUniformBuffer
        | AccessType::ComputeShaderReadSampledImageOrUniformTexelBuffer
        | AccessType::ComputeShaderReadOther
        | AccessType::AnyShaderReadUniformBuffer
        | AccessType::AnyShaderReadUniformBufferOrVertexBuffer
        | AccessType::AnyShaderReadSampledImageOrUniformTexelBuffer
        | AccessType::AnyShaderReadOther
        | AccessType::TransferRead
        | AccessType::HostRead
        | AccessType::Present
        | AccessType::RayTracingShaderReadSampledImageOrUniformTexelBuffer
        | AccessType::RayTracingShaderReadColorInputAttachment
        | AccessType::RayTracingShaderReadDepthStencilInputAttachment
        | AccessType::RayTracingShaderReadAccelerationStructure
        | AccessType::RayTracingShaderReadOther => true,

        _ => false,
    }
}
pub fn is_hazard(prev_accesses: &[AccessType], next_accesses: &[AccessType]) -> bool {
    if prev_accesses.is_empty() || next_accesses.is_empty() {
        return false;
    }
    //WAR, RAW, WAW are hazards
    let prev_read_only = prev_accesses
        .iter()
        .fold(true, |acc, access| acc & is_read(access));
    let next_read_only = next_accesses
        .iter()
        .fold(true, |acc, access| acc & is_read(access));

    //RARs are Ok
    !(prev_read_only && next_read_only)
}
