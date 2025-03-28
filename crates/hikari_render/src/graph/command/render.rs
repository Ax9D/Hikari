use ash::vk::{self};

use crate::buffer::Buffer;
use crate::graph::graphics::pipeline::*;
use crate::image::SampledImage;
use crate::{IndexType, PhysicalRenderpass};

use super::{CommandBuffer, PipelineLookup, update_shader};

pub struct RenderpassBeginInfo<'a> {
    pub renderpass: &'a PhysicalRenderpass,
    pub area: vk::Rect2D,
    pub framebuffer: vk::Framebuffer,
}

pub struct PassRecordInfo {
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
}

pub struct RenderpassCommands<'cmd, 'graph> {
    cmd: &'cmd mut CommandBuffer<'graph>,
    renderpass: &'cmd PhysicalRenderpass,
    pipeline_ctx: PipelineContext,
}

impl<'cmd, 'graph> RenderpassCommands<'cmd, 'graph> {
    pub fn new(
        cmd: &'cmd mut CommandBuffer<'graph>,
        begin_info: RenderpassBeginInfo<'cmd>,
    ) -> Self {
        hikari_dev::profile_function!();
        let device = cmd.device;
        unsafe {
            hikari_dev::profile_scope!("vkBeginRenderPass");
            debug_assert!(begin_info.renderpass.pass != vk::RenderPass::null());

            let begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(begin_info.renderpass.pass)
                .render_area(begin_info.area)
                .framebuffer(begin_info.framebuffer)
                .clear_values(&begin_info.renderpass.clear_values);

            device
                .raw()
                .cmd_begin_render_pass(cmd.raw(), &begin_info, vk::SubpassContents::INLINE);
        }

        cmd.saved_state.descriptor_state.set_all_dirty();

        let pipeline_ctx = PipelineContext::default();
        Self {
            cmd,
            renderpass: begin_info.renderpass,
            pipeline_ctx,
        }
    }
    #[inline]
    pub(crate) fn inner(&mut self) -> &mut &'cmd mut CommandBuffer<'graph> {
        &mut self.cmd
    }
    #[inline]
    pub fn raw(&self) -> vk::CommandBuffer {
        self.cmd.raw()
    }
    #[inline]
    pub fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.cmd.set_viewport(x, y, width, height);
    }
    #[inline]
    pub fn set_scissor(&mut self, offset_x: i32, offset_y: i32, width: u32, height: u32) {
        self.cmd.set_scissor(offset_x, offset_y, width, height);
    }
    pub fn set_shader(&mut self, shader: &Arc<crate::Shader>) {
        self.pipeline_ctx.set_shader(shader, &mut self.cmd.saved_state.descriptor_state.dirty_sets)
    }
    pub fn set_pipeline_state(&mut self, pipeline_state: PipelineState) {
        let pctx = &mut self.pipeline_ctx;
        pctx.psv.pipeline_state = pipeline_state;

        pctx.pipeline_dirty = true;
    }
    pub fn set_vertex_input_layout(&mut self, input_layout: VertexInputLayout) {
        let pctx = &mut self.pipeline_ctx;
        pctx.psv.pipeline_state.input_layout = input_layout;

        pctx.pipeline_dirty = true;
    }
    pub fn set_primitive_topology(&mut self, primitive_topology: PrimitiveTopology) {
        let pctx = &mut self.pipeline_ctx;
        pctx.psv.pipeline_state.primitive_topology = primitive_topology;

        pctx.pipeline_dirty = true;
    }
    pub fn set_depth_stencil_state(&mut self, depth_stencil_state: DepthStencilState) {
        let pctx = &mut self.pipeline_ctx;
        pctx.psv.pipeline_state.depth_stencil_state = depth_stencil_state;

        pctx.pipeline_dirty = true;
    }
    pub fn set_rasterizer_state(&mut self, rasterizer_state: RasterizerState) {
        let pctx = &mut self.pipeline_ctx;
        pctx.psv.pipeline_state.rasterizer_state = rasterizer_state;

        pctx.pipeline_dirty = true;
    }
    pub fn set_blend_state(&mut self, blend_state: BlendState) {
        let pctx = &mut self.pipeline_ctx;
        pctx.psv.pipeline_state.blend_state = blend_state;

        pctx.pipeline_dirty = true;
    }

    pub fn get_pipeline_state(&self) -> &PipelineState {
        &self.pipeline_ctx.psv.pipeline_state
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
    #[deprecated(note = "use `set_buffer` instead")]
    pub fn set_uniform_buffer<B: Buffer>(
        &mut self,
        buffer: &B,
        span: Range<usize>,
        set: u32,
        binding: u32,
    ) {
        self.set_buffer(buffer, span, set, binding)
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
        self.cmd.set_buffer(buffer, span, set, binding)
    }
    pub fn set_bindless(&mut self, set: vk::DescriptorSet) {
        self.cmd.set_bindless(set)
    }
    pub fn set_vertex_buffer<B: Buffer>(&mut self, buffer: &B, binding: u32) {
        unsafe {
            self.cmd.device.raw().cmd_bind_vertex_buffers(
                self.cmd.raw(),
                binding,
                &[buffer.buffer()],
                &[0],
            );
        }
    }
    pub fn set_vertex_buffers<const N: usize>(&mut self, buffers: &[&dyn Buffer; N], binding: u32) {
        let vk_buffers = buffers.map(|buffer| buffer.buffer());
        let offsets = [0; N];

        unsafe {
            self.cmd.device.raw().cmd_bind_vertex_buffers(
                self.cmd.raw(),
                binding,
                &vk_buffers,
                &offsets,
            );
        }
    }
    pub fn set_index_buffer<B: Buffer + IndexType>(&mut self, buffer: &B) {
        unsafe {
            self.cmd.device.raw().cmd_bind_index_buffer(
                self.cmd.raw(),
                buffer.buffer(),
                0,
                B::index_type(),
            );
        }
    }
    pub fn push_constants<T: Copy>(&mut self, data: &T, offset: usize) {
        self.cmd
            .saved_state
            .descriptor_state
            .push_constants(data, offset);
    }
    pub fn draw(&mut self, vertices: Range<usize>, instances: Range<usize>) {
        hikari_dev::profile_function!();
        let cmd = self.cmd.raw();

        self.flush_render_state();

        unsafe {
            hikari_dev::profile_scope!("vkCmdDraw");
            self.cmd.device.raw().cmd_draw(
                cmd,
                vertices.len() as u32,
                instances.len() as u32,
                vertices.start as u32,
                instances.start as u32,
            );
        }
    }
    pub fn draw_indexed(
        &mut self,
        indices: Range<usize>,
        base_vertex: i32,
        instances: Range<usize>,
    ) {
        hikari_dev::profile_function!();
        let cmd = self.cmd.raw();
        
        self.flush_render_state();

        unsafe {
            hikari_dev::profile_scope!("vkCmdDrawIndexed");
            self.cmd.device.raw().cmd_draw_indexed(
                cmd,
                indices.len() as u32,
                instances.len() as u32,
                indices.start as u32,
                base_vertex,
                instances.start as u32,
            );
        }
    }
    pub fn begin_debug_region(&mut self, name: impl AsRef<str>, color: hikari_math::Vec4) {
        self.cmd.begin_debug_region(name, color)
    }
    pub fn end_debug_region(&mut self) {
        self.cmd.end_debug_region()
    }
    fn flush_render_state(&mut self) {
        hikari_dev::profile_function!();

        let cmd = self.cmd.raw();
        let pipeline_lookup = &mut self.cmd.saved_state.pipeline_lookup;
        let descriptor_pool = &mut self.cmd.saved_state.descriptor_pool;
        let renderpass = self.renderpass;

        self.pipeline_ctx
            .flush(self.cmd.device, cmd, renderpass, pipeline_lookup);

        self.cmd.saved_state.descriptor_state.flush(
            self.cmd.device,
            cmd,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_ctx
                .psv
                .shader
                .as_ref()
                .expect("No shader was bound"),
            descriptor_pool,
        );
    }
}

impl<'cmd, 'graph> Drop for RenderpassCommands<'cmd, 'graph> {
    fn drop(&mut self) {
        unsafe {
            self.cmd.device.raw().cmd_end_render_pass(self.cmd.raw());
        }
    }
}

use std::ops::Range;
use std::sync::Arc;
use crate::graph::pass::graphics::pipeline::PipelineState;

#[derive(Hash, PartialEq, Eq, Clone, Default)]
pub struct PipelineStateVector {
    pub shader: Option<Arc<crate::Shader>>,
    pub pipeline_state: PipelineState,
}

#[derive(Default)]
struct PipelineContext {
    psv: PipelineStateVector,
    pipeline_dirty: bool,
}
impl PipelineContext {
    pub fn new() -> Self {
        let psv = PipelineStateVector {
            shader: None,
            pipeline_state: PipelineState::default(),
        };

        Self {
            psv,
            pipeline_dirty: false,
        }
    }
    pub fn set_shader(&mut self, new: &Arc<crate::Shader>, dirty_sets: &mut u32) {
        self.pipeline_dirty = update_shader(&mut self.psv.shader, new, dirty_sets);
    }
    pub fn flush(
        &mut self,
        device: &Arc<crate::Device>,
        cmd: vk::CommandBuffer,
        renderpass: &PhysicalRenderpass,
        pipe_lookup: &mut PipelineLookup,
    ) {
        //self.descriptor_state.flush(device, cmd, vk::PipelineBindPoint::GRAPHICS, &self.psv.shader, descriptor_pool);
        if self.pipeline_dirty {
            hikari_dev::profile_scope!("Flushing Render Pipeline");
            let vk_pipeline = pipe_lookup
                .get_vk_graphics_pipeline(
                    &self.psv,
                    renderpass.pass,
                    renderpass.n_color_attachments,
                )
                .expect("Failed to create Pipeline");
            unsafe {
                hikari_dev::profile_scope!("Bind Pipeline");
                device.raw().cmd_bind_pipeline(
                    cmd,
                    vk::PipelineBindPoint::GRAPHICS,
                    vk_pipeline,
                );
            }
            self.pipeline_dirty = false;
        }
    }
}
