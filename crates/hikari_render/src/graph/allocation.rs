use std::{collections::HashMap, sync::Arc};

use ash::{prelude::VkResult, vk};
use vec_map::VecMap;
use vk_sync_fork::AccessType;

use crate::{graph::pass::AttachmentKind, renderpass::PhysicalRenderpass, texture::SampledImage};

use super::{pass::AnyPass, resources::GraphResources, Renderpass};

unsafe impl Sync for BarrierStorage {}
unsafe impl Send for BarrierStorage {}

pub struct BarrierStorage {
    image_barriers: Vec<vk::ImageMemoryBarrier2KHR>,
}
impl BarrierStorage {
    pub fn new() -> Self {
        Self {
            image_barriers: Vec::new(),
        }
    }
    pub fn add_image_barrier(
        &mut self,
        image: &SampledImage,
        previous_accesses: &[AccessType],
        next_accesses: &[AccessType],
        queue_index: u32,
    ) {
        let range = vk::ImageSubresourceRange {
            aspect_mask: crate::texture::sampled_image::format_to_aspect_flags(
                image.config().format,
            ),
            base_mip_level: 0,
            level_count: image.config().mip_levels,
            base_array_layer: 0,
            layer_count: 1,
        };

        use vk_sync_fork as sync;
        let barrier = sync::ImageBarrier {
            previous_accesses,
            next_accesses,
            previous_layout: sync::ImageLayout::Optimal,
            next_layout: sync::ImageLayout::Optimal,
            discard_contents: false,
            src_queue_family_index: queue_index,
            dst_queue_family_index: queue_index,
            image: image.image(),
            range,
        };
        let (
            src_stage_mask,
            dst_stage_mask,
            vk::ImageMemoryBarrier {
                src_access_mask,
                dst_access_mask,
                old_layout,
                new_layout,
                src_queue_family_index,
                dst_queue_family_index,
                image,
                subresource_range,
                ..
            },
        ) = sync::get_image_memory_barrier(&barrier);

        log::info!(
            "old_layout {:?} src_access_mask {:?}",
            old_layout,
            src_access_mask
        );
        log::info!(
            "new_layout {:?} dst_access_mask {:?}",
            new_layout,
            dst_access_mask
        );

        log::info!("\n");

        use crate::barrier;

        let barrier = *vk::ImageMemoryBarrier2KHR::builder()
            .image(image)
            .subresource_range(subresource_range)
            .src_access_mask(barrier::to_sync2_access_flags(src_access_mask))
            .dst_access_mask(barrier::to_sync2_access_flags(dst_access_mask))
            .src_stage_mask(barrier::to_sync2_stage_flags(src_stage_mask))
            .dst_stage_mask(barrier::to_sync2_stage_flags(dst_stage_mask))
            .old_layout(old_layout)
            .new_layout(new_layout);

        self.image_barriers.push(barrier);
    }
    pub unsafe fn apply(&self, device: &Arc<crate::Device>, cmd: vk::CommandBuffer) {
        let dependency_info = vk::DependencyInfoKHR::builder()
            .image_memory_barriers(&self.image_barriers)
            .dependency_flags(vk::DependencyFlags::BY_REGION);

        device
            .extensions()
            .synchronization2
            .cmd_pipeline_barrier2(cmd, &dependency_info);
    }
}

pub struct AllocationData {
    device: Arc<crate::Device>,
    framebuffers: VecMap<vk::Framebuffer>, //Pass ix to framebuffer
    barriers: VecMap<BarrierStorage>,      //Pass ix to BarrierStorage
    renderpasses: VecMap<PhysicalRenderpass>, //Pass ix to AllocatedRenderpass
}

impl AllocationData {
    pub fn new<T: crate::Args>(
        device: &Arc<crate::Device>,
        passes: &[AnyPass<T>],
        resources: &GraphResources,
    ) -> VkResult<Self> {
        let mut alloc = Self {
            device: device.clone(),
            framebuffers: VecMap::new(),
            barriers: VecMap::new(),
            renderpasses: VecMap::new(),
        };

        alloc.create_barriers(device, passes, resources);

        for (ix, pass) in passes.iter().enumerate() {
            if let AnyPass::Render(pass) = pass {
                if !pass.present_to_swapchain {
                    alloc.create_renderpass(device, pass, ix, resources)?;
                    alloc.allocate_framebuffers(device, pass, ix, resources)?;
                }
            }
        }

        Ok(alloc)
    }
    pub fn resize_framebuffers<T: crate::Args>(
        &mut self,
        device: &Arc<crate::Device>,
        passes: &[AnyPass<T>],
        resources: &GraphResources,
    ) -> VkResult<()> {
        self.framebuffers = VecMap::new();
        self.barriers = VecMap::new();

        self.create_barriers(device, passes, resources);
        for (ix, pass) in passes.iter().enumerate() {
            if let AnyPass::Render(pass) = pass {
                if !pass.present_to_swapchain {
                    //self.create_renderpass(device, pass, ix, resources)?;
                    self.allocate_framebuffers(device, pass, ix, resources)?;
                }
            }
        }

        Ok(())
    }
    fn create_renderpass<T: crate::Args>(
        &mut self,
        device: &Arc<crate::Device>,
        pass: &Renderpass<T>,
        ix: usize,
        graph_resources: &GraphResources,
    ) -> VkResult<()> {
        let mut depth_attachment_ref = None;

        let max_attachment_location = pass.outputs().iter().fold(0, |acc, output| match output {
            crate::graph::pass::Output::DrawImage(_, config) => match config.kind {
                AttachmentKind::Color(location) => acc.max(location),
                _ => acc,
            },
            _ => acc,
        });
        let mut color_attachment_refs = vec![
            vk::AttachmentReference {
                attachment: vk::ATTACHMENT_UNUSED,
                layout: vk::ImageLayout::UNDEFINED
            };
            max_attachment_location as usize + 1
        ];

        let mut attachments = Vec::new();
        let mut clear_values = Vec::new();

        for output in pass.outputs() {
            if let super::pass::Output::DrawImage(handle, attachment_config) = output {
                let image = graph_resources.get_image(handle).unwrap();

                let (final_layout, clear_value) = match attachment_config.kind {
                    AttachmentKind::Color(location) => {
                        color_attachment_refs[location as usize] =
                            *vk::AttachmentReference::builder()
                                .attachment(attachments.len() as u32)
                                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

                        (
                            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                            vk::ClearValue {
                                color: vk::ClearColorValue {
                                    float32: [0.0, 0.0, 0.0, 0.0],
                                },
                            },
                        )
                    }
                    AttachmentKind::DepthStencil => {
                        depth_attachment_ref.replace(
                            *vk::AttachmentReference::builder()
                                .attachment(attachments.len() as u32)
                                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
                        );

                        (
                            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                            vk::ClearValue {
                                depth_stencil: vk::ClearDepthStencilValue {
                                    depth: 1.0,
                                    stencil: 0,
                                },
                            },
                        )
                    }
                    AttachmentKind::DepthOnly => {
                        depth_attachment_ref.replace(
                            *vk::AttachmentReference::builder()
                                .attachment(attachments.len() as u32)
                                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
                        );

                        (
                            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                            vk::ClearValue {
                                depth_stencil: vk::ClearDepthStencilValue {
                                    depth: 1.0,
                                    stencil: 0,
                                },
                            },
                        )
                    }
                };

                let pass_barriers = self.barriers.get(ix).unwrap();

                let initial_layout = pass_barriers
                    .image_barriers
                    .iter()
                    .find_map(|barrier| {
                        if barrier.image == image.image() {
                            Some(barrier.new_layout)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(vk::ImageLayout::UNDEFINED);
                clear_values.push(clear_value);
                let attachment = *vk::AttachmentDescription::builder()
                    .format(image.config().format)
                    .load_op(attachment_config.load_op)
                    .store_op(attachment_config.store_op)
                    .stencil_store_op(attachment_config.stencil_store_op)
                    .stencil_load_op(attachment_config.stencil_load_op)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .initial_layout(initial_layout)
                    .final_layout(final_layout);

                attachments.push(attachment);
            }
        }

        let mut subpass_desc = *vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs);

        if let Some(depth_stencil_attachment_ref) = &depth_attachment_ref {
            subpass_desc.p_depth_stencil_attachment = depth_stencil_attachment_ref as *const _;
        }

        let subpass_descs = [subpass_desc];
        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpass_descs);

        let pass = unsafe { device.raw().create_render_pass(&create_info, None)? };

        log::debug!("Created renderpass");

        let n_color_attachments = color_attachment_refs.len();

        self.renderpasses.insert(
            ix,
            PhysicalRenderpass {
                pass,
                n_color_attachments,
                clear_values,
            },
        );

        Ok(())
    }
    fn allocate_framebuffers<T: crate::Args>(
        &mut self,
        device: &Arc<crate::Device>,
        pass: &Renderpass<T>,
        ix: usize,
        graph_resources: &GraphResources,
    ) -> VkResult<()> {
        let mut image_handles = Vec::new();
        for output in pass.outputs() {
            if let super::pass::Output::DrawImage(handle, attachment_config) = output {
                image_handles.push(handle.clone());
            }
        }

        let renderpass = &self.renderpasses[ix];

        let framebuffer = super::framebuffer::from_allocation_data(
            device,
            graph_resources,
            &image_handles,
            renderpass.pass,
        )?;

        if self.framebuffers.insert(ix, framebuffer).is_some() {
            panic!("Framebuffer with same index already exists");
        }

        Ok(())
    }
    fn create_barriers<T: crate::Args>(
        &mut self,
        device: &Arc<crate::Device>,
        passes: &[AnyPass<T>],
        graph_resources: &GraphResources,
    ) {
        let mut prev_accesses: HashMap<_, Vec<AccessType>> = graph_resources
            .image_handles()
            .map(|handle| (handle, Vec::new()))
            .collect();

        for (ix, pass) in passes.iter().enumerate() {
            let mut current_accesses: HashMap<_, Vec<_>> = HashMap::new();
            let mut barrier_storage = BarrierStorage::new();

            for input in pass.inputs() {
                match input {
                    crate::graph::pass::Input::ReadImage(handle, access)
                    | crate::graph::pass::Input::SampleImage(handle, access, _) => current_accesses
                        .entry(handle.clone())
                        .or_default()
                        .push(*access),
                }
            }
            for output in pass.outputs() {
                let (handle, access) = match output {
                    crate::graph::pass::Output::WriteImage(handle, access) => {
                        (handle.clone(), *access)
                    }
                    crate::graph::pass::Output::DrawImage(handle, config) => {
                        (handle.clone(), config.access)
                    }
                    crate::graph::pass::Output::StorageBuffer => {
                        continue;
                    }
                };

                current_accesses
                    .entry(handle.clone())
                    .or_default()
                    .push(access)
            }
            for (handle, current_accesses) in current_accesses {
                let prev_accesses = prev_accesses.get_mut(&handle).unwrap();

                if crate::barrier::is_hazard(prev_accesses, &current_accesses) {
                    //Add Transition
                    barrier_storage.add_image_barrier(
                        graph_resources.get_image(&handle).unwrap(),
                        prev_accesses,
                        &current_accesses,
                        device.unified_queue_ix,
                    );
                }

                let old_accesses = std::mem::replace(prev_accesses, current_accesses);
            }

            if self.barriers.insert(ix, barrier_storage).is_some() {
                panic!("Barrier with same index already exists");
            }
        }
    }
    pub fn get_framebuffer(&self, ix: usize) -> vk::Framebuffer {
        self.framebuffers[ix]
    }
    pub fn get_renderpass(&self, ix: usize) -> &PhysicalRenderpass {
        &self.renderpasses[ix]
    }
    pub fn get_barrier_storage(&self, ix: usize) -> &BarrierStorage {
        &self.barriers[ix]
    }
}

impl Drop for AllocationData {
    fn drop(&mut self) {
        let device = self.device.raw();
        unsafe {
            for (_, pass) in &self.renderpasses {
                device.destroy_render_pass(pass.pass, None);
            }

            for (_, &fb) in &self.framebuffers {
                device.destroy_framebuffer(fb, None);
            }
        }
    }
}
