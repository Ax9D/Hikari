use std::{ops::Range, sync::Arc};

use ash::vk;

use crate::{Buffer, CommandBuffer, SampledImage};

use super::PipelineLookup;

pub struct ComputepassCommands<'cmd, 'graph> {
    cmd: &'cmd mut CommandBuffer<'graph>,
    pipeline_ctx: PipelineContext,
}

struct PipelineContext {
    shader: Option<Arc<crate::Shader>>,
    pipeline_dirty: bool,
}
impl PipelineContext {
    pub fn new() -> Self {
        Self {
            shader: None,
            pipeline_dirty: false,
        }
    }
    pub fn set_shader(&mut self, shader: &Arc<crate::Shader>) -> Option<Arc<crate::Shader>> {
        self.shader.replace(shader.clone())
    }
    pub fn flush(
        &mut self,
        device: &Arc<crate::Device>,
        cmd: vk::CommandBuffer,
        pipeline_lookup: &mut PipelineLookup,
    ) {
        hikari_dev::profile_function!();
        if self.pipeline_dirty {
            if let Some(shader) = &self.shader {
                let vk_pipeline = pipeline_lookup
                    .get_vk_compute_pipeline(&shader)
                    .expect("Failed to create Pipeline");
                unsafe {
                    hikari_dev::profile_scope!("Bind Pipeline");
                    device.raw().cmd_bind_pipeline(
                        cmd,
                        vk::PipelineBindPoint::COMPUTE,
                        vk_pipeline,
                    );
                }
            }
            self.pipeline_dirty = false;
        }
    }
}
impl<'cmd, 'graph> ComputepassCommands<'cmd, 'graph> {
    pub fn new(cmd: &'cmd mut CommandBuffer<'graph>) -> Self {
        Self {
            cmd,
            pipeline_ctx: PipelineContext::new(),
        }
    }
    pub fn raw(&mut self) -> vk::CommandBuffer {
        self.cmd.raw()
    }
    #[inline]
    pub(crate) fn inner(&mut self) -> &mut &'cmd mut CommandBuffer<'graph> {
        &mut self.cmd
    }
    #[inline]
    pub fn set_image(&mut self, image: &SampledImage, set: u32, binding: u32) {
        self.cmd.set_image(image, set, binding)
    }
    #[inline]
    pub fn set_image_mip(&mut self, image: &SampledImage, mip_level: u32, set: u32, binding: u32) {
        self.cmd.set_image_mip(image, mip_level, set, binding)
    }
    #[inline]
    pub fn set_image_view_and_sampler(
        &mut self,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
        set: u32,
        binding: u32,
        index: usize,
    ) {
        self.cmd
            .set_image_view_and_sampler(image_view, sampler, set, binding, index)
    }
    #[inline]
    pub fn set_image_array(&mut self, image: &SampledImage, set: u32, binding: u32, index: usize) {
        self.cmd.set_image_array(image, set, binding, index)
    }
    #[inline]
    pub fn set_image_mip_array(
        &mut self,
        image: &SampledImage,
        mip_level: u32,
        set: u32,
        binding: u32,
        index: usize,
    ) {
        self.cmd
            .set_image_mip_array(image, mip_level, set, binding, index)
    }
    #[inline]
    pub fn apply_image_barrier(
        &mut self,
        image: &SampledImage,
        previous_accesses: &[crate::vk_sync::AccessType],
        next_accesses: &[crate::vk_sync::AccessType],
        previous_layout: crate::vk_sync::ImageLayout,
        next_layout: crate::vk_sync::ImageLayout,
        range: vk::ImageSubresourceRange,
    ) {
        self.cmd.apply_image_barrier(
            image,
            previous_accesses,
            next_accesses,
            previous_layout,
            next_layout,
            range,
        )
    }
    pub fn set_buffer<B: Buffer>(
        &mut self,
        buffer: &B,
        span: Range<usize>,
        set: u32,
        binding: u32,
    ) {
        self.cmd.set_buffer(buffer, span, set, binding);
    }
    pub fn set_shader(&mut self, shader: &Arc<crate::Shader>) {
        if let Some(old_shader) = self.pipeline_ctx.set_shader(shader) {
            if old_shader.pipeline_layout() == shader.pipeline_layout() {
                return;
            }
        }
        //self.cmd.saved_state.descriptor_state.dirty_sets = shader.pipeline_layout().set_mask();
        self.pipeline_ctx.pipeline_dirty = true;
    }
    pub fn push_constants<T: Copy>(&mut self, data: &T, offset: usize) {
        self.cmd
            .saved_state
            .descriptor_state
            .push_constants(data, offset);
    }
    pub fn dispatch(&mut self, n_workgroups: (u32, u32, u32)) {
        hikari_dev::profile_function!();
        let cmd = self.cmd.raw();

        self.flush_compute_state();

        unsafe {
            self.cmd
                .device
                .raw()
                .cmd_dispatch(cmd, n_workgroups.0, n_workgroups.1, n_workgroups.2);
        }
    }
    pub fn begin_debug_region(&mut self, name: impl AsRef<str>, color: hikari_math::Vec4) {
        self.cmd.begin_debug_region(name, color)
    }
    pub fn end_debug_region(&mut self) {
        self.cmd.end_debug_region()
    }
    pub fn copy_image(&self, src: &SampledImage, src_layout: vk::ImageLayout, dst: &SampledImage, dst_layout: vk::ImageLayout, copy_info: &[vk::ImageCopy]) {
        self.cmd.copy_image(src, src_layout, dst, dst_layout, copy_info)
    }
    fn flush_compute_state(&mut self) {
        hikari_dev::profile_function!();

        let cmd = self.cmd.raw();
        let pipeline_lookup = &mut self.cmd.saved_state.pipeline_lookup;
        let descriptor_pool = &mut self.cmd.saved_state.descriptor_pool;

        self.pipeline_ctx
            .flush(self.cmd.device, cmd, pipeline_lookup);

        self.cmd.saved_state.descriptor_state.flush(
            self.cmd.device,
            cmd,
            vk::PipelineBindPoint::COMPUTE,
            self.pipeline_ctx
                .shader
                .as_ref()
                .expect("No shader was bound"),
            descriptor_pool,
        );
    }
}
