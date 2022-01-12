use ash::vk;

use crate::buffer::Buffer;
use crate::graph::graphics::pipeline::*;
use crate::texture::SampledImage;
use crate::{IndexType, PhysicalRenderpass};

use super::CommandBuffer;

pub struct RenderpassBeginInfo<'a> {
    pub renderpass: &'a PhysicalRenderpass,
    pub area: vk::Rect2D,
    pub framebuffer: vk::Framebuffer,
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
        let device = cmd.device;
        unsafe {
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

        Self {
            cmd,
            renderpass: begin_info.renderpass,
            pipeline_ctx: PipelineContext::default(),
        }
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
        if let Some(old_shader) = self.pipeline_ctx.set_shader(shader) {
            if old_shader.pipeline_layout() == shader.pipeline_layout() {
                return;
            }
        }

        self.pipeline_ctx.pipeline_dirty = true;
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
        self.cmd.set_image(image, set, binding);
    }
    pub fn set_uniform_buffer<B: Buffer>(
        &mut self,
        buffer: &B,
        span: Range<usize>,
        set: u32,
        binding: u32,
    ) {
        self.cmd.set_uniform_buffer(buffer, span, set, binding);
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

        let pipeline_lookup = &mut self.cmd.saved_state.pipeline_lookup;
        let descriptor_pool = &mut self.cmd.saved_state.descriptor_pool;
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

        let pipeline_lookup = &mut self.cmd.saved_state.pipeline_lookup;
        let descriptor_pool = &mut self.cmd.saved_state.descriptor_pool;
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

    fn flush_render_state(&mut self) {
        hikari_dev::profile_function!();

        let cmd = self.cmd.raw();
        let pipeline_lookup = &mut self.cmd.saved_state.pipeline_lookup;
        let descriptor_pool = &mut self.cmd.saved_state.descriptor_pool;
        let renderpass = &self.renderpass;

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

use ash::prelude::VkResult;

use crate::Shader;

use crate::graph::pass::graphics::pipeline::PipelineState;
use crate::util::CacheMap;

#[derive(Hash, PartialEq, Eq, Clone, Default)]
pub struct PipelineStateVector {
    pub shader: Option<Arc<crate::Shader>>,
    pub pipeline_state: PipelineState,
}

pub struct PipelineLookup {
    device: Arc<crate::Device>,
    vk_pipeline_cache: vk::PipelineCache,
    pipelines: CacheMap<PipelineStateVector, vk::Pipeline>,
}

impl PipelineLookup {
    pub fn new(
        device: &Arc<crate::Device>,
        capacity: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            device: device.clone(),
            vk_pipeline_cache: device.pipeline_cache(),
            pipelines: CacheMap::new(capacity),
        })
    }
    fn create_pipeline(
        &self,
        shader: &Shader,
        pipeline_state: &PipelineState,
        vk_renderpass: vk::RenderPass,
        n_color_attachments: usize,
    ) -> VkResult<vk::Pipeline> {
        Ok(unsafe {
            let pipeline = pipeline_state.create_pipeline(
                &self.device,
                shader,
                vk_renderpass,
                n_color_attachments,
            );

            log::debug!("Created new pipeline {:?}", pipeline);

            pipeline
        })
    }
    fn destroy_pipeline(device: &Arc<crate::Device>, vk_pipeline: vk::Pipeline) {
        unsafe {
            device.raw().destroy_pipeline(vk_pipeline, None);
            log::debug!("Destroyed pipeline: {:?}", vk_pipeline);
        }
    }
    pub fn get_vk_pipeline(
        &mut self,
        pipeline_state_vector: &PipelineStateVector,
        renderpass: vk::RenderPass,
        n_color_attachments: usize,
    ) -> VkResult<vk::Pipeline> {
        let device = &self.device;
        let pipeline = self.pipelines.get(pipeline_state_vector, |psv| unsafe {
            Ok(psv.pipeline_state.create_pipeline(
                device,
                psv.shader.as_ref().expect("Shader must not be None"),
                renderpass,
                n_color_attachments,
            ))
        })?;

        Ok(*pipeline)
    }

    //Call once per frame
    pub fn new_frame(&mut self) {
        let device = &self.device;
        self.pipelines
            .unused()
            .drain(..)
            .for_each(|pipeline| Self::destroy_pipeline(device, pipeline));
    }
}

impl Drop for PipelineLookup {
    fn drop(&mut self) {
        self.new_frame();
    }
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
    pub fn set_shader(&mut self, shader: &Arc<Shader>) -> Option<Arc<Shader>> {
        if let Some(current_shader) = self.psv.shader.as_mut() {
            let old_shader = std::mem::replace(current_shader, shader.clone());

            return Some(old_shader);
        }

        self.psv.shader.replace(shader.clone())
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
            if self.psv.shader.is_some() {
                let vk_pipeline = pipe_lookup
                    .get_vk_pipeline(&self.psv, renderpass.pass, renderpass.n_color_attachments)
                    .expect("Failed to create Pipeline");
                unsafe {
                    hikari_dev::profile_scope!("Bind Pipeline");
                    device.raw().cmd_bind_pipeline(
                        cmd,
                        vk::PipelineBindPoint::GRAPHICS,
                        vk_pipeline,
                    );
                }
            }
            self.pipeline_dirty = false;
        }
    }
}
