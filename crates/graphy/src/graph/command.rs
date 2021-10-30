use std::sync::Arc;

use ash::vk::{self, MAX_DESCRIPTION_SIZE, RasterizationOrderAMD};
use bytemuck::Pod;
use fxhash::FxHashMap;

use crate::{Shader, descriptor::{DescriptorPool, DescriptorSetAllocator, DescriptorSetLayout, DescriptorSetState, MAX_DESCRIPTOR_SETS}, texture::SampledImage};

use super::{AllocationData, ResourceBindInfo, graphics::pipeline::{BlendState, DepthStencilState, PipelineState, PrimitiveTopology, RasterizerState}, pass, pipeline::{PipelineLookup, PipelineStateVector}};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandExecutionError {
    #[error("No pipeline is currently bound")]
    UnboundPipeline,
    #[error("Unknown set or binding")]
    UnknownSetOrBinding
}
const PUSH_CONSTANT_SIZE: usize = 128;
#[derive(Clone, Copy)]
struct DescriptorState {
    sets: [DescriptorSetState; MAX_DESCRIPTOR_SETS],
    push_constant_data: [u8; PUSH_CONSTANT_SIZE],
    dirty_sets: u32,
    push_constant_dirty: bool,
}

impl DescriptorState {
    pub fn new(shader: &Arc<Shader>) -> Self {
        let mut sets = [DescriptorSetState::default(); MAX_DESCRIPTOR_SETS];

        for set in 0..MAX_DESCRIPTOR_SETS {
            sets[set].set = set as u32;
            sets[set].set_layout = shader.pipeline_layout().set_layouts()[set];
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
    pub fn set_image(&mut self, shader: &Arc<Shader>, image_view: vk::ImageView, sampler: vk::Sampler, set: u32, binding: u32, ix: usize) -> Result<(), CommandExecutionError>{
        if  Self::set_and_binding_exists(shader, shader.pipeline_layout().set_layouts()[set as usize].combined_image_sampler_mask(), set, binding) {
            self.dirty_sets |= 1 << set;

            Ok( self.sets[set as usize].set_image(binding, ix, image_view, sampler) )
        }
        else {
            Err(CommandExecutionError::UnknownSetOrBinding)
        } 
    }
    pub fn set_uniform_buffer(&mut self, shader: &Arc<Shader>, buffer: vk::Buffer, offset: vk::DeviceSize, range: vk::DeviceSize, set: u32, binding: u32, ix: usize) -> Result<(), CommandExecutionError> {
        if  Self::set_and_binding_exists(shader, shader.pipeline_layout().set_layouts()[set as usize].uniform_buffer_mask(), set, binding) {
            self.dirty_sets |= 1 << set;

            Ok( self.sets[set as usize].set_buffer(binding, buffer, offset, range) )
        }
        else {
            Err(CommandExecutionError::UnknownSetOrBinding)
        } 
    }
    pub fn push_constants<T: Pod>(&mut self, shader: &Arc<Shader>, data: &T) {
        let byte_slice = bytemuck::bytes_of(data);
        self.push_constant_data[0..byte_slice.len()].copy_from_slice(byte_slice);
    }
    pub fn flush(&mut self, device: &Arc<crate::Device>, cmd: vk::CommandBuffer, bind_point: vk::PipelineBindPoint, shader: &Arc<Shader>, descriptor_pool: &mut DescriptorPool) {
        crate::util::for_each_bit(self.dirty_sets, 0..MAX_DESCRIPTOR_SETS, |set| {
            let set_layout = &shader.pipeline_layout().set_layouts()[set as usize];

            let mut allocator = descriptor_pool.get(set_layout);
            let state = &self.sets[set as usize];
            let set = allocator.get(state);
            let sets = [set];

            unsafe { 
                device.raw().cmd_bind_descriptor_sets(cmd, bind_point, shader.pipeline_layout().raw(), 0, &sets, &[]);
            }
        });

        unsafe {
            device.raw().cmd_push_constants(cmd, shader.pipeline_layout().raw(), shader.pipeline_layout().push_constant_stage_flags(), 0, &self.push_constant_data);
        }

        self.dirty_sets = 0;
        self.push_constant_dirty = false;
    }
}
struct PipelineContext {
    psv: PipelineStateVector,
    pipeline_dirty: bool,
    descriptor_state: DescriptorState,
}
impl PipelineContext {
    pub fn new(shader: &Arc<Shader>) -> Self {
        let psv = PipelineStateVector {
            shader: shader.clone(),
            pipeline_state: PipelineState::default()
        };

        let descriptor_state = DescriptorState::new(shader);

        Self {
            psv,
            pipeline_dirty: true,
            descriptor_state
        }
    }
    pub fn set_shader(&mut self, shader: &Arc<Shader>) {
        if self.psv.shader.hash == shader.hash {
            return ;
        }

        let old_shader = std::mem::replace(&mut self.psv.shader, shader.clone());
        
        if shader.pipeline_layout() == old_shader.pipeline_layout() {
            return ;
        }

        self.descriptor_state = DescriptorState::new(shader);

    }
    pub fn flush(&mut self, renderpass: vk::RenderPass, pipe_lookup: &mut PipelineLookup, descriptor_pool: &mut DescriptorPool) {

        if self.pipeline_dirty {
            pipe_lookup.get_vk_pipeline(&self.psv, renderpass, 1).unwrap();
        }

    }
}

impl Drop for PipelineContext {
    fn drop(&mut self) {
        todo!()
    }
}

pub struct CommandBufferSavedState<'a> {
    pipeline_lookup: &'a mut PipelineLookup,
    descriptor_pool: &'a mut DescriptorPool
}
pub struct CommandBuffer<'a> {
    device: &'a Arc<crate::Device>,
    saved_state: CommandBufferSavedState<'a>,
    cmd: vk::CommandBuffer,
    pipeline_context: Option<PipelineContext>,
    active_renderpass: Option<vk::RenderPass>,
}

impl<'a> CommandBuffer<'a> {
    pub(crate) fn new(
        device: &'a Arc<crate::Device>,
        cmd: vk::CommandBuffer,
        saved_state: CommandBufferSavedState<'a>,
    ) -> Self {
        Self {
            device,
            cmd,
            pipeline_context: None,
            saved_state,
            active_renderpass: None
        }
    }
    pub fn raw(&self) -> vk::CommandBuffer {
        self.cmd
    }
    pub(crate) fn begin_renderpass(&mut self, begin_info: &vk::RenderPassBeginInfo) {
        unsafe {
            debug_assert!(begin_info.render_pass!=vk::RenderPass::null());

            self.active_renderpass = Some(begin_info.render_pass);
            self.device.raw().cmd_begin_render_pass(self.raw(), begin_info, vk::SubpassContents::INLINE);
        }
    }
    pub(crate) fn end_renderpass(&mut self) {
        if let Some(renderpass) = self.active_renderpass {
            unsafe {
                self.device.raw().cmd_end_render_pass(self.raw());
            }

            self.active_renderpass = None;
        } else {
            panic!("Renderpass was never started");
        }
    }
    fn try_get_pctx(&self) -> Result<&PipelineContext, CommandExecutionError> {
        match &self.pipeline_context {
            Some(pctx) => Ok(pctx),
            None => Err(CommandExecutionError::UnboundPipeline),
        } 
    }
    fn try_get_pctx_mut(&mut self) -> Result<&mut PipelineContext, CommandExecutionError> {
        match &mut self.pipeline_context {
            Some(pctx) => Ok(pctx),
            None => Err(CommandExecutionError::UnboundPipeline),
        } 
    }
    pub fn set_shader(&mut self, shader: &Arc<crate::Shader>) {

        match self.try_get_pctx_mut() {
            Ok(pctx) => pctx.set_shader(shader),
            Err(_) => {
                self.pipeline_context = Some(PipelineContext::new(shader));
            },
        }
    }
    pub fn set_pipeline_state(&mut self, pipeline_state: PipelineState) -> Result<(), CommandExecutionError>{
        let pctx = self.try_get_pctx_mut()?;
        pctx.psv.pipeline_state = pipeline_state;

        pctx.pipeline_dirty = true;

        Ok(())
    }
    pub fn set_primitive_topology(&mut self, primitive_topology: PrimitiveTopology) -> Result<(), CommandExecutionError> {
        let pctx = self.try_get_pctx_mut()?;
        pctx.psv.pipeline_state.primitive_topology = primitive_topology;

        pctx.pipeline_dirty = true;

        Ok(())
    }
    pub fn set_depth_stencil_state(&mut self, depth_stencil_state: DepthStencilState) -> Result<(), CommandExecutionError> {
        let pctx = self.try_get_pctx_mut()?;
        pctx.psv.pipeline_state.depth_stencil_state = depth_stencil_state;

        pctx.pipeline_dirty = true;

        Ok(())
    }
    pub fn set_rasterizer_state(&mut self, rasterizer_state: RasterizerState) -> Result<(), CommandExecutionError> {
        let pctx = self.try_get_pctx_mut()?;
        pctx.psv.pipeline_state.rasterizer_state = rasterizer_state;

        pctx.pipeline_dirty = true;

        Ok(())
    }
    pub fn set_blend_state(&mut self, blend_state: BlendState) -> Result<(), CommandExecutionError> {
        let pctx = self.try_get_pctx_mut()?;
        pctx.psv.pipeline_state.blend_state = blend_state;

        pctx.pipeline_dirty = true;

        Ok(())
    }

    pub fn get_pipeline_state(&self) -> Result<&PipelineState, CommandExecutionError> {
        Ok(&self.try_get_pctx()?.psv.pipeline_state)
    }
    pub(crate) fn set_image(&mut self, image: &SampledImage, set: u32, binding: u32) -> Result<(), CommandExecutionError>{
        let pctx = self.try_get_pctx()?;
        pctx.descriptor_state.set_image(&pctx.psv.shader, image.image_view(0).unwrap(), image.sampler(), set, binding, 0)
    }
    pub fn push_constants<T: Pod>(&mut self, data: &T) -> Result<(), CommandExecutionError> {
        let pctx = self.try_get_pctx_mut()?;
        pctx.descriptor_state.push_constants(&pctx.psv.shader, data);

        Ok(())
    }
    pub fn draw(&mut self) {

    }
}
