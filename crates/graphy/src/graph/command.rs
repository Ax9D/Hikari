use std::{ops::Range, sync::Arc};

use ash::{vk::{self, RasterizationOrderAMD, MAX_DESCRIPTION_SIZE}, prelude::VkResult};
use bytemuck::Pod;
use fxhash::FxHashMap;

use crate::{
    descriptor::{
        DescriptorPool, DescriptorSetAllocator, DescriptorSetLayout, DescriptorSetState,
        MAX_DESCRIPTOR_SETS,
    },
    texture::SampledImage,
    Shader,
};

use super::{
    graphics::pipeline::{
        BlendState, DepthStencilState, PipelineState, PrimitiveTopology, RasterizerState,
    },
    pass,
    pipeline::{PipelineLookup, PipelineStateVector},
    AllocationData, CompiledRenderpass, ResourceBindInfo,
};

use thiserror::Error;

const PUSH_CONSTANT_SIZE: usize = 128;
#[derive(Clone, Copy)]
struct DescriptorState {
    sets: [DescriptorSetState; MAX_DESCRIPTOR_SETS],
    push_constant_data: [u8; PUSH_CONSTANT_SIZE],
    dirty_sets: u32,
    push_constant_dirty: bool,
}

impl Default for DescriptorState {
    fn default() -> Self {
        Self {
            sets: Default::default(),
            push_constant_data: [0; PUSH_CONSTANT_SIZE],
            dirty_sets: 0,
            push_constant_dirty: false,
        }
    }
}

impl DescriptorState {
    pub fn new(shader: &Arc<Shader>) -> Self {
        let mut sets = [DescriptorSetState::default(); MAX_DESCRIPTOR_SETS];

        for set in 0..MAX_DESCRIPTOR_SETS {
            sets[set].set = set as u32;
            //sets[set].set_layout = shader.pipeline_layout().set_layouts()[set];
        }

        Self {
            sets,
            dirty_sets: 0,
            push_constant_data: [0; PUSH_CONSTANT_SIZE],
            push_constant_dirty: false,
        }
    }
    fn set_and_binding_exists(shader: &Arc<Shader>, mask: u32, set: u32, binding: u32) -> bool {
        let set_exists = shader.pipeline_layout().set_mask() & 1 << set == 1;
        let binding_exists = mask & 1 << binding == 1;

        set_exists && binding_exists
    }
    pub fn set_image(
        &mut self,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
        set: u32,
        binding: u32,
        ix: usize,
    ) {
        self.dirty_sets |= 1 << set;

        self.sets[set as usize].set_image(binding, ix, image_view, sampler);
    }
    pub fn set_uniform_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        range: vk::DeviceSize,
        set: u32,
        binding: u32,
        ix: usize,
    ) {
        self.dirty_sets |= 1 << set;

        self.sets[set as usize].set_buffer(binding, buffer, offset, range);
    }
    pub fn push_constants<T: Pod>(&mut self, data: &T) {
        let byte_slice = bytemuck::bytes_of(data);
        self.push_constant_data[0..byte_slice.len()].copy_from_slice(byte_slice);
    }
    pub fn reset(&mut self) {
        for set in &mut self.sets {
            set.reset();
        }
    }
    pub fn flush(
        &mut self,
        device: &Arc<crate::Device>,
        cmd: vk::CommandBuffer,
        bind_point: vk::PipelineBindPoint,
        shader: &Arc<Shader>,
        descriptor_pool: &mut DescriptorPool,
    ) {
        crate::util::for_each_bit(self.dirty_sets, 0..MAX_DESCRIPTOR_SETS, |set| {
            let set_layout = &shader.pipeline_layout().set_layouts()[set as usize];

            let allocator = descriptor_pool.get(set_layout);
            let state = &self.sets[set as usize];

            //assert!(set_layout == &state.set_layout);
            
            let set = allocator.get(state);
            let sets = [set];

            unsafe {
                device.raw().cmd_bind_descriptor_sets(
                    cmd,
                    bind_point,
                    shader.pipeline_layout().raw(),
                    0,
                    &sets,
                    &[],
                );
            }
        });

        if self.push_constant_dirty {
            unsafe {
                device.raw().cmd_push_constants(
                    cmd,
                    shader.pipeline_layout().raw(),
                    shader.pipeline_layout().push_constant_stage_flags(),
                    0,
                    &self.push_constant_data,
                );
            }

            self.push_constant_dirty = false;
        }

        self.dirty_sets = 0;
    }
}
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
        match self.psv.shader.as_mut() {
            Some(current_shader) => {
                let old_shader = std::mem::replace(current_shader, shader.clone());

                return Some(old_shader);
            }
            None => {}
        }

        self.psv.shader.replace(shader.clone())
    }
    pub fn flush(
        &mut self,
        device: &Arc<crate::Device>,
        cmd: vk::CommandBuffer,
        renderpass: &CompiledRenderpass,
        pipe_lookup: &mut PipelineLookup,
        descriptor_pool: &mut DescriptorPool,
    ) {
        //self.descriptor_state.flush(device, cmd, vk::PipelineBindPoint::GRAPHICS, &self.psv.shader, descriptor_pool);

        if self.pipeline_dirty {
            if self.psv.shader.is_some() {
                let vk_pipeline = pipe_lookup
                    .get_vk_pipeline(&self.psv, renderpass.inner, renderpass.n_color_attachments)
                    .unwrap();
                unsafe {
                    device.raw().cmd_bind_pipeline(
                        cmd,
                        vk::PipelineBindPoint::GRAPHICS,
                        vk_pipeline,
                    );
                }
            }
        }
    }
}
pub struct CommandBufferSavedState<'a> {
    pub pipeline_lookup: &'a mut PipelineLookup,
    pub descriptor_pool: &'a mut DescriptorPool,
}
pub struct CommandBuffer<'a> {
    device: &'a Arc<crate::Device>,
    saved_state: CommandBufferSavedState<'a>,
    cmd: vk::CommandBuffer,
    pipeline_context: PipelineContext,
    descriptor_state: DescriptorState,
    active_renderpass: Option<CompiledRenderpass>,
}

impl<'a> CommandBuffer<'a> {
    pub(crate) fn from_existing(
        device: &'a Arc<crate::Device>,
        cmd: vk::CommandBuffer,
        saved_state: CommandBufferSavedState<'a>,
    ) -> Self {
        Self {
            device,
            cmd,
            pipeline_context: PipelineContext::new(),
            saved_state,
            active_renderpass: None,
            descriptor_state: DescriptorState::default(),
        }
    }
    pub fn raw(&self) -> vk::CommandBuffer {
        self.cmd
    }
    pub(crate) fn begin_renderpass(
        &mut self,
        renderpass: CompiledRenderpass,
        begin_info: &vk::RenderPassBeginInfo,
    ) {
        unsafe {
            debug_assert!(begin_info.render_pass != vk::RenderPass::null());
            debug_assert!(renderpass.inner == begin_info.render_pass);

            self.active_renderpass = Some(renderpass);
            self.device.raw().cmd_begin_render_pass(
                self.raw(),
                begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }
    pub(crate) fn end_renderpass(&mut self) {

        if self.active_renderpass.is_none() {
            log::error!("No renderpass was started");
        }

        unsafe {
            self.device.raw().cmd_end_render_pass(self.raw());
        }

        self.active_renderpass = None;
    }
    fn pctx(&self) -> &PipelineContext {
        &self.pipeline_context
    }
    fn pctx_mut(&mut self) -> &mut PipelineContext {
        &mut self.pipeline_context
    }
    pub fn set_shader(&mut self, shader: &Arc<crate::Shader>) {
        if let Some(old_shader) = self.pipeline_context.set_shader(shader) {
            if old_shader.pipeline_layout() == shader.pipeline_layout() {
                return;
            }
        }

        self.descriptor_state = DescriptorState::new(shader);
    }
    pub fn set_pipeline_state(&mut self, pipeline_state: PipelineState) {
        let pctx = self.pctx_mut();
        pctx.psv.pipeline_state = pipeline_state;

        pctx.pipeline_dirty = true;
    }
    pub fn set_primitive_topology(&mut self, primitive_topology: PrimitiveTopology) {
        let pctx = self.pctx_mut();
        pctx.psv.pipeline_state.primitive_topology = primitive_topology;

        pctx.pipeline_dirty = true;
    }
    pub fn set_depth_stencil_state(&mut self, depth_stencil_state: DepthStencilState) {
        let pctx = self.pctx_mut();
        pctx.psv.pipeline_state.depth_stencil_state = depth_stencil_state;

        pctx.pipeline_dirty = true;
    }
    pub fn set_rasterizer_state(&mut self, rasterizer_state: RasterizerState) {
        let pctx = self.pctx_mut();
        pctx.psv.pipeline_state.rasterizer_state = rasterizer_state;

        pctx.pipeline_dirty = true;
    }
    pub fn set_blend_state(&mut self, blend_state: BlendState) {
        let pctx = self.pctx_mut();
        pctx.psv.pipeline_state.blend_state = blend_state;

        pctx.pipeline_dirty = true;
    }

    pub fn get_pipeline_state(&self) -> &PipelineState {
        &self.pctx().psv.pipeline_state
    }
    pub(crate) fn set_image(&mut self, image: &SampledImage, set: u32, binding: u32) {
        self.descriptor_state.set_image(
            image.image_view(0).unwrap(),
            image.sampler(),
            set,
            binding,
            0,
        );
    }
    pub fn push_constants<T: Pod>(&mut self, data: &T) {
        self.descriptor_state.push_constants(data);
    }
    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        let pipeline_lookup = &mut self.saved_state.pipeline_lookup;
        let descriptor_pool = &mut self.saved_state.descriptor_pool;
        let cmd = self.cmd;

        self.flush_render_state();

        unsafe {
            self.device.raw().cmd_draw_indexed(
                cmd,
                indices.len() as u32,
                instances.len() as u32,
                indices.start,
                base_vertex,
                instances.start,
            );
        }
    }

    fn flush_render_state(&mut self) {
        let pipeline_lookup = &mut self.saved_state.pipeline_lookup;
        let descriptor_pool = &mut self.saved_state.descriptor_pool;
        let renderpass = self.active_renderpass.as_ref().unwrap();
        let cmd = self.cmd;

        self.pipeline_context.flush(
            &self.device,
            cmd,
            renderpass,
            pipeline_lookup,
            descriptor_pool,
        );

        self.descriptor_state.flush(&self.device, cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline_context.psv.shader.as_ref().unwrap(), descriptor_pool);
    }

    pub(crate) fn reset(&mut self) -> VkResult<()>{
        self.descriptor_state.reset();

        unsafe {
            self.device.raw().reset_command_buffer(
                self.raw(),
                vk::CommandBufferResetFlags::empty(),
            )
        }
    }
}
