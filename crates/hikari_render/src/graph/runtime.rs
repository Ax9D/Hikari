use std::sync::Arc;

use ash::{prelude::VkResult, vk};

use crate::{
    descriptor::DescriptorPool,
    graph::{
        command::{compute::ComputepassCommands, render::PassRecordInfo, CommandBufferSavedState},
        CommandBuffer,
    },
    swapchain::Swapchain,
    ComputePass,
};

use super::{
    allocation::AllocationData,
    command::{DescriptorState, PipelineLookup},
    graphics::Renderpass,
    pass::AnyPass,
    resources::GraphResources,
};
struct FrameData {
    pub render_finished_semaphore: vk::Semaphore,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_fence: vk::Fence,

    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
}
impl FrameData {
    pub fn new(device: &Arc<crate::Device>) -> VkResult<Self> {
        unsafe {
            let create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(device.unified_queue_ix)
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

            let command_pool = device.raw().create_command_pool(&create_info, None)?;

            let create_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .command_buffer_count(1)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffer = device.raw().allocate_command_buffers(&create_info)?[0];

            let create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

            let render_finished_fence = device.raw().create_fence(&create_info, None)?;

            let create_info =
                vk::SemaphoreCreateInfo::builder().flags(vk::SemaphoreCreateFlags::empty());

            let render_semaphore = device.raw().create_semaphore(&create_info, None)?;
            let present_semaphore = device.raw().create_semaphore(&create_info, None)?;

            Ok(Self {
                render_finished_semaphore: render_semaphore,
                image_available_semaphore: present_semaphore,
                render_finished_fence,
                command_pool,
                command_buffer,
            })
        }
    }
    pub unsafe fn delete(&self, device: &Arc<crate::Device>) {
        device
            .raw()
            .wait_for_fences(&[self.render_finished_fence], true, 1000000000)
            .unwrap();

        device.raw().destroy_command_pool(self.command_pool, None);
        device.raw().destroy_fence(self.render_finished_fence, None);
        device
            .raw()
            .destroy_semaphore(self.render_finished_semaphore, None);
        device
            .raw()
            .destroy_semaphore(self.image_available_semaphore, None);

        log::debug!("Deleted Framedata");
    }
}

struct FrameState {
    frame_number: usize,
    frames: [FrameData; 2],
}

impl FrameState {
    pub fn new(device: &Arc<crate::Device>) -> VkResult<Self> {
        Ok(Self {
            frame_number: 1,
            frames: [FrameData::new(device)?, FrameData::new(device)?],
        })
    }

    #[inline]
    pub fn current_frame(&self) -> &FrameData {
        &self.frames[(self.frame_number % 2)]
    }
    #[inline]
    pub fn last_frame(&self) -> &FrameData {
        &self.frames[(self.frame_number.wrapping_sub(1) % 2)]
    }
    #[inline]
    pub fn current_frame_number(&self) -> usize {
        self.frame_number
    }
    #[inline]
    pub fn update(&mut self) {
        self.frame_number = self.frame_number.wrapping_add(1);
    }

    pub unsafe fn delete(&self, device: &Arc<crate::Device>) {
        for frame in &self.frames {
            frame.delete(device);
        }
    }
}

pub struct ExecutionContext {}
pub struct GraphExecutor {
    device: Arc<crate::Device>,
    descriptor_pool: DescriptorPool,
    pipeline_lookup: PipelineLookup,
    descriptor_state: DescriptorState,

    frame_state: FrameState,
}

impl GraphExecutor {
    pub fn new(device: &Arc<crate::Device>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            device: device.clone(),
            descriptor_pool: DescriptorPool::new(device),
            pipeline_lookup: PipelineLookup::new(device, 100)?,
            descriptor_state: DescriptorState::new(),
            frame_state: FrameState::new(device)?,
        })
    }
    pub fn finish(&mut self) -> VkResult<()> {
        Self::finish_internal(&self.device, &mut self.frame_state)
    }
    fn finish_internal(device: &Arc<crate::Device>, frame_state: &mut FrameState) -> VkResult<()> {
        unsafe {
            hikari_dev::profile_scope!("Waiting on GPU");
            let fences = &[frame_state.last_frame().render_finished_fence];
            device.raw().wait_for_fences(fences, true, 5_000_000_000)?;
        }

        Ok(())
    }
    pub fn execute_and_present<T: crate::Args>(
        &mut self,
        args: <T::Ref as crate::ByRef>::Item,
        size: (u32, u32),
        passes: &mut [AnyPass<T>],
        resources: &GraphResources,
        allocation_data: &AllocationData,
        swapchain: &mut Swapchain,
    ) -> VkResult<()> {
        hikari_dev::profile_function!();
        //self.finish()?;

        //log::debug!("Reset fences");
        let current_frame = self.frame_state.current_frame();

        let device = &self.device;
        let mut cmd = CommandBuffer::from_existing(
            device,
            current_frame.command_buffer,
            CommandBufferSavedState {
                pipeline_lookup: &mut self.pipeline_lookup,
                descriptor_pool: &mut self.descriptor_pool,
                descriptor_state: &mut self.descriptor_state,
            },
        );

        cmd.reset()?;
        cmd.begin()?;

        let swapchain_image_ix = swapchain
            .acquire_next_image_ix(
                5_000_000_000,
                self.frame_state.current_frame().image_available_semaphore,
                vk::Fence::null(),
            )
            .expect("Swapchain image");

        for (ix, pass) in passes.iter_mut().enumerate() {
            match pass {
                AnyPass::Render(pass) => {
                    hikari_dev::profile_scope!(pass.name());

                    //log::debug!("Executing pass {}", pass.name());
                    Self::execute_renderpass(
                        device,
                        &mut cmd,
                        args,
                        size,
                        ix,
                        pass,
                        resources,
                        allocation_data,
                        Some((swapchain, swapchain_image_ix)),
                    )?;
                }
                AnyPass::Compute(pass) => {
                    hikari_dev::profile_scope!(pass.name());
                    Self::execute_computepass(
                        device,
                        &mut cmd,
                        args,
                        size,
                        ix,
                        pass,
                        resources,
                        allocation_data,
                    )?;
                }
            }
        }

        cmd.end()?;

        Self::finish_internal(&self.device, &mut self.frame_state).expect("Finish internal");

        Self::submit_and_present(
            device,
            &self.frame_state,
            &cmd,
            swapchain,
            swapchain_image_ix,
        )
        .expect("Submit");
        self.frame_state.update();
        self.descriptor_pool.new_frame();
        self.pipeline_lookup.new_frame();

        Ok(())
    }
    pub fn execute<T: crate::Args>(
        &mut self,
        args: <T::Ref as crate::ByRef>::Item,
        size: (u32, u32),
        passes: &mut [AnyPass<T>],
        resources: &GraphResources,
        allocation_data: &AllocationData,
    ) -> VkResult<()> {
        //self.finish()?;

        //log::debug!("Reset fences");
        let current_frame = self.frame_state.current_frame();

        let device = &self.device;
        let mut cmd = CommandBuffer::from_existing(
            device,
            current_frame.command_buffer,
            CommandBufferSavedState {
                pipeline_lookup: &mut self.pipeline_lookup,
                descriptor_pool: &mut self.descriptor_pool,
                descriptor_state: &mut self.descriptor_state,
            },
        );

        cmd.reset()?;
        cmd.begin()?;

        for (ix, pass) in passes.iter_mut().enumerate() {
            match pass {
                AnyPass::Render(pass) => {
                    hikari_dev::profile_scope!(pass.name());

                    //log::debug!("Executing pass {}", pass.name());
                    Self::execute_renderpass(
                        device,
                        &mut cmd,
                        args,
                        size,
                        ix,
                        pass,
                        resources,
                        allocation_data,
                        None,
                    )?;
                }
                AnyPass::Compute(pass) => {
                    hikari_dev::profile_scope!(pass.name());
                    Self::execute_computepass(
                        device,
                        &mut cmd,
                        args,
                        size,
                        ix,
                        pass,
                        resources,
                        allocation_data,
                    )?;
                }
            }
        }

        cmd.end()?;
        Self::finish_internal(&self.device, &mut self.frame_state)?;

        Self::submit(
            device,
            &cmd,
            &[],
            &[],
            self.frame_state.current_frame().render_finished_fence,
        )?;

        self.frame_state.update();
        self.descriptor_pool.new_frame();
        self.pipeline_lookup.new_frame();

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_renderpass<'cmd, 'graph, T: crate::Args>(
        device: &Arc<crate::Device>,
        cmd: &'cmd mut CommandBuffer<'graph>,
        args: <T::Ref as crate::ByRef>::Item,
        size: (u32, u32),
        ix: usize,
        pass: &mut Renderpass<T>,
        resources: &GraphResources,
        allocation_data: &AllocationData,
        swapchain_data: Option<(&mut Swapchain, u32)>,
    ) -> VkResult<()> {
        hikari_dev::profile_function!();
        let barriers = allocation_data.get_barrier_storage(ix);

        unsafe {
            barriers.apply(device, cmd.raw());
        }

        if pass.record_fn.is_some() {
            let (vk_pass, framebuffer) = if pass.present_to_swapchain {
                let (swapchain, image_ix) = swapchain_data.expect("Swapchain not provided");
                (
                    swapchain.renderpass(),
                    swapchain.framebuffers()[image_ix as usize],
                )
            } else {
                (
                    allocation_data.get_renderpass(ix),
                    allocation_data.get_framebuffer(ix),
                )
            };

            let (width, height) = pass.render_area.get_physical_size_2d(size);
            let area = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D { width, height },
            };

            let mut rcmd = cmd.begin_renderpass(super::command::RenderpassBeginInfo {
                renderpass: vk_pass,
                area,
                framebuffer,
            });

            //Self::bind_resources::<T>(rcmd.inner(), resources, pass.inputs(), pass.outputs());

            //rcmd.set_viewport(0.0, 0.0, width as f32, height as f32);
            //rcmd.set_scissor(0, 0, width, height);

            let record_info = PassRecordInfo {
                framebuffer_width: width,
                framebuffer_height: height,
            };

            (pass.record_fn.as_mut().unwrap())(&mut rcmd, resources, &record_info, args);
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn execute_computepass<'cmd, 'graph, T: crate::Args>(
        device: &Arc<crate::Device>,
        cmd: &'cmd mut CommandBuffer<'graph>,
        args: <T::Ref as crate::ByRef>::Item,
        size: (u32, u32),
        ix: usize,
        pass: &mut ComputePass<T>,
        resources: &GraphResources,
        allocation_data: &AllocationData,
    ) -> VkResult<()> {
        hikari_dev::profile_function!();
        let barriers = allocation_data.get_barrier_storage(ix);
        unsafe {
            barriers.apply(device, cmd.raw());
        }
        let mut ccmd = ComputepassCommands::new(cmd);
        if pass.record_fn.is_some() {
            //Self::bind_resources::<T>(ccmd.inner(), resources, pass.inputs(), pass.outputs());

            let record_info = PassRecordInfo {
                framebuffer_width: size.0,
                framebuffer_height: size.1,
            };
            (pass.record_fn.as_mut().unwrap())(&mut ccmd, resources, &record_info, args);
        }

        Ok(())
    }
    // fn bind_resources<'cmd, 'graph, T: crate::Args>(cmd: &mut&'cmd mut CommandBuffer<'graph>, resources: &GraphResources, inputs: &[Input], outputs: &[Output]) {
    //     //log::debug!("Binding renderpass resources");
    //     for input in inputs {
    //         match input {
    //             crate::graph::pass::Input::SampleImage(handle, _, binding, index) |
    //             crate::graph::pass::Input::ReadStorageImage(handle, _, binding, index ) => {
    //                 let image = resources.get_image(handle).unwrap();

    //                 cmd.set_image_array(image, 0, *binding, *index);
    //             }
    //             crate::graph::pass::Input::ReadStorageBuffer(handle, _, binding) => {
    //                 let buffer = resources.get_dyn_buffer(handle).unwrap();
    //                 cmd.set_buffer(buffer, 0..buffer.len(), 0, *binding);
    //             }
    //             _ => {}
    //         }
    //     }
    //     for output in outputs {
    //         match output {
    //             crate::graph::pass::Output::WriteStorageImage(handle, _, binding) => {
    //                 let image = resources.get_image(handle).unwrap();

    //                 cmd.set_image_array(image, 0, *binding, 0);
    //             }
    //             crate::graph::pass::Output::WriteStorageBuffer(handle, _, binding) => {
    //                 let buffer = resources.get_dyn_buffer(handle).unwrap();
    //                 cmd.set_buffer(buffer, 0..buffer.len(), 0, *binding);
    //             }
    //             _=> {}
    //         }
    //     }
    // }
    fn submit(
        device: &Arc<crate::Device>,
        cmd: &CommandBuffer,
        wait_semaphores: &[vk::Semaphore],
        signal_semaphores: &[vk::Semaphore],
        fence: vk::Fence,
    ) -> VkResult<()> {
        unsafe {
            device.raw().reset_fences(&[fence])?;
        }
        let cbs = [cmd.raw()];
        let submit_info = vk::SubmitInfo::builder()
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .wait_semaphores(wait_semaphores)
            .signal_semaphores(signal_semaphores)
            .command_buffers(&cbs);

        let result = device.graphics_queue_submit(&[*submit_info], fence);
        match result {
            Ok(_) => {}
            Err(err) => {
                assert!(err != vk::Result::ERROR_DEVICE_LOST && err != vk::Result::NOT_READY);
            }
        }
        Ok(())
    }
    fn submit_and_present(
        device: &Arc<crate::Device>,
        frame_state: &FrameState,
        cmd: &CommandBuffer,
        swapchain: &mut Swapchain,
        image_ix: u32,
    ) -> VkResult<()> {
        hikari_dev::profile_function!();
        //Wait till image is available again after previous presentation
        let wait_semaphores = [frame_state.current_frame().image_available_semaphore];
        //Signal end of render so that the swapchain can present
        let signal_semaphores = [frame_state.current_frame().render_finished_semaphore];

        Self::submit(
            device,
            cmd,
            &wait_semaphores,
            &signal_semaphores,
            frame_state.current_frame().render_finished_fence,
        )?;
        match swapchain.present(
            image_ix,
            frame_state.current_frame().render_finished_semaphore,
        ) {
            Ok(suboptimal) => {
                if suboptimal {
                    log::warn!("Swapchain suboptimal");
                }
            }
            Err(err) => {
                if err == vk::Result::ERROR_OUT_OF_DATE_KHR || err == vk::Result::NOT_READY {
                    log::warn!("Swapchain out of date");
                }
            }
        };

        Ok(())
    }
}

impl Drop for GraphExecutor {
    fn drop(&mut self) {
        log::debug!("Dropping FrameState");
        unsafe {
            self.device.raw().device_wait_idle().unwrap();
            self.frame_state.delete(&self.device);
        }
    }
}
