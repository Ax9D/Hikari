use ash::vk;
use vk_sync_fork::AccessType;

#[allow(clippy::too_many_arguments)]
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
    matches!(
        access,
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
            | AccessType::RayTracingShaderReadOther
    )
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

    if prev_accesses == next_accesses && prev_read_only && next_read_only {
        return false;
    }

    return true;
}
/// This only exists because vk_sync doesn't expose this type
pub struct AccessInfo {
    pub stage_mask: vk::PipelineStageFlags,
    pub access_mask: vk::AccessFlags,
    pub image_layout: vk::ImageLayout
}

pub fn get_access_info(access_type: AccessType) -> AccessInfo {
    match access_type {
        AccessType::Nothing => AccessInfo {
            stage_mask: vk::PipelineStageFlags::empty(),
            access_mask: vk::AccessFlags::empty(),
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::CommandBufferReadNVX => AccessInfo {
            stage_mask: vk::PipelineStageFlags::COMMAND_PREPROCESS_NV,
            access_mask: vk::AccessFlags::COMMAND_PREPROCESS_READ_NV,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::IndirectBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::DRAW_INDIRECT,
            access_mask: vk::AccessFlags::INDIRECT_COMMAND_READ,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::IndexBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::VERTEX_INPUT,
            access_mask: vk::AccessFlags::INDEX_READ,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::VertexBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::VERTEX_INPUT,
            access_mask: vk::AccessFlags::VERTEX_ATTRIBUTE_READ,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::VertexShaderReadUniformBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::VERTEX_SHADER,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::VertexShaderReadSampledImageOrUniformTexelBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::VERTEX_SHADER,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
        AccessType::VertexShaderReadOther => AccessInfo {
            stage_mask: vk::PipelineStageFlags::VERTEX_SHADER,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::TessellationControlShaderReadUniformBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER,
            access_mask: vk::AccessFlags::UNIFORM_READ,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::TessellationControlShaderReadSampledImageOrUniformTexelBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
        AccessType::TessellationControlShaderReadOther => AccessInfo {
            stage_mask: vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::TessellationEvaluationShaderReadUniformBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER,
            access_mask: vk::AccessFlags::UNIFORM_READ,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::TessellationEvaluationShaderReadSampledImageOrUniformTexelBuffer => {
            AccessInfo {
                stage_mask: vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER,
                access_mask: vk::AccessFlags::SHADER_READ,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            }
        }
        AccessType::TessellationEvaluationShaderReadOther => AccessInfo {
            stage_mask: vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::GeometryShaderReadUniformBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::GEOMETRY_SHADER,
            access_mask: vk::AccessFlags::UNIFORM_READ,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::GeometryShaderReadSampledImageOrUniformTexelBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::GEOMETRY_SHADER,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
        AccessType::GeometryShaderReadOther => AccessInfo {
            stage_mask: vk::PipelineStageFlags::GEOMETRY_SHADER,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::FragmentShaderReadUniformBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
            access_mask: vk::AccessFlags::UNIFORM_READ,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
        AccessType::FragmentShaderReadColorInputAttachment => AccessInfo {
            stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
            access_mask: vk::AccessFlags::INPUT_ATTACHMENT_READ,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
        AccessType::FragmentShaderReadDepthStencilInputAttachment => AccessInfo {
            stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
            access_mask: vk::AccessFlags::INPUT_ATTACHMENT_READ,
            image_layout: vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
        },
        AccessType::FragmentShaderReadOther => AccessInfo {
            stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::ColorAttachmentRead => AccessInfo {
            stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ,
            image_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        },
        AccessType::DepthStencilAttachmentRead => AccessInfo {
            stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
            image_layout: vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
        },
        AccessType::ComputeShaderReadUniformBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::COMPUTE_SHADER,
            access_mask: vk::AccessFlags::UNIFORM_READ,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::ComputeShaderReadSampledImageOrUniformTexelBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::COMPUTE_SHADER,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
        AccessType::ComputeShaderReadOther => AccessInfo {
            stage_mask: vk::PipelineStageFlags::COMPUTE_SHADER,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::AnyShaderReadUniformBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::ALL_COMMANDS,
            access_mask: vk::AccessFlags::UNIFORM_READ,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::AnyShaderReadUniformBufferOrVertexBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::ALL_COMMANDS,
            access_mask: vk::AccessFlags::UNIFORM_READ | vk::AccessFlags::VERTEX_ATTRIBUTE_READ,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::AnyShaderReadSampledImageOrUniformTexelBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::ALL_COMMANDS,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
        AccessType::AnyShaderReadOther => AccessInfo {
            stage_mask: vk::PipelineStageFlags::ALL_COMMANDS,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::TransferRead => AccessInfo {
            stage_mask: vk::PipelineStageFlags::TRANSFER,
            access_mask: vk::AccessFlags::TRANSFER_READ,
            image_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        },
        AccessType::HostRead => AccessInfo {
            stage_mask: vk::PipelineStageFlags::HOST,
            access_mask: vk::AccessFlags::HOST_READ,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::Present => AccessInfo {
            stage_mask: vk::PipelineStageFlags::empty(),
            access_mask: vk::AccessFlags::empty(),
            image_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        },
        AccessType::CommandBufferWriteNVX => AccessInfo {
            stage_mask: vk::PipelineStageFlags::COMMAND_PREPROCESS_NV,
            access_mask: vk::AccessFlags::COMMAND_PREPROCESS_WRITE_NV,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::VertexShaderWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::VERTEX_SHADER,
            access_mask: vk::AccessFlags::SHADER_WRITE,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::TessellationControlShaderWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER,
            access_mask: vk::AccessFlags::SHADER_WRITE,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::TessellationEvaluationShaderWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER,
            access_mask: vk::AccessFlags::SHADER_WRITE,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::GeometryShaderWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::GEOMETRY_SHADER,
            access_mask: vk::AccessFlags::SHADER_WRITE,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::FragmentShaderWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
            access_mask: vk::AccessFlags::SHADER_WRITE,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::ColorAttachmentWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            image_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        },
        AccessType::DepthStencilAttachmentWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            image_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        },
        AccessType::DepthAttachmentWriteStencilReadOnly => AccessInfo {
            stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
            image_layout: vk::ImageLayout::DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL,
        },
        AccessType::StencilAttachmentWriteDepthReadOnly => AccessInfo {
            stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
            image_layout: vk::ImageLayout::DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL,
        },
        AccessType::ComputeShaderWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::COMPUTE_SHADER,
            access_mask: vk::AccessFlags::SHADER_WRITE,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::AnyShaderWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::ALL_COMMANDS,
            access_mask: vk::AccessFlags::SHADER_WRITE,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::TransferWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::TRANSFER,
            access_mask: vk::AccessFlags::TRANSFER_WRITE,
            image_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        },
        AccessType::HostWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::HOST,
            access_mask: vk::AccessFlags::HOST_WRITE,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::ColorAttachmentReadWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            image_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        },
        AccessType::General => AccessInfo {
            stage_mask: vk::PipelineStageFlags::ALL_COMMANDS,
            access_mask: vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::RayTracingShaderReadSampledImageOrUniformTexelBuffer => AccessInfo {
            stage_mask: vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
        AccessType::RayTracingShaderReadColorInputAttachment => AccessInfo {
            stage_mask: vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR,
            access_mask: vk::AccessFlags::INPUT_ATTACHMENT_READ,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
        AccessType::RayTracingShaderReadDepthStencilInputAttachment => AccessInfo {
            stage_mask: vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR,
            access_mask: vk::AccessFlags::INPUT_ATTACHMENT_READ,
            image_layout: vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
        },
        AccessType::RayTracingShaderReadAccelerationStructure => AccessInfo {
            stage_mask: vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR,
            access_mask: vk::AccessFlags::ACCELERATION_STRUCTURE_READ_KHR,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::RayTracingShaderReadOther => AccessInfo {
            stage_mask: vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR,
            access_mask: vk::AccessFlags::SHADER_READ,
            image_layout: vk::ImageLayout::GENERAL,
        },
        AccessType::AccelerationStructureBuildWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            access_mask: vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::AccelerationStructureBuildRead => AccessInfo {
            stage_mask: vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            access_mask: vk::AccessFlags::ACCELERATION_STRUCTURE_READ_KHR,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
        AccessType::AccelerationStructureBufferWrite => AccessInfo {
            stage_mask: vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            access_mask: vk::AccessFlags::TRANSFER_WRITE,
            image_layout: vk::ImageLayout::UNDEFINED,
        },
    }
}