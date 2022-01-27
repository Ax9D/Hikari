use std::{ops::Range, sync::Arc};

use ash::{prelude::VkResult, vk};

use crate::{
    buffer::Buffer,
    descriptor::{DescriptorPool, DescriptorSetState, MAX_DESCRIPTOR_SETS},
    texture::SampledImage,
    Shader,
};

use self::render::PipelineLookup;

pub mod compute;
pub mod render;

pub use render::RenderpassBeginInfo;
pub use render::RenderpassCommands;

const PUSH_CONSTANT_SIZE: usize = 128;
const PUSH_CONSTANT_SIZEU32: usize = PUSH_CONSTANT_SIZE / std::mem::size_of::<u32>();

#[derive(Clone, Copy)]
pub struct DescriptorState {
    sets: [DescriptorSetState; MAX_DESCRIPTOR_SETS],
    push_constant_data: [u8; PUSH_CONSTANT_SIZE],
    dirty_sets: u32,
    push_constant_update_range: Option<(usize, usize)>,
}

impl Default for DescriptorState {
    fn default() -> Self {
        Self {
            sets: Default::default(),
            push_constant_data: [0; PUSH_CONSTANT_SIZE],
            dirty_sets: 0,
            push_constant_update_range: None,
        }
    }
}

impl DescriptorState {
    pub fn new() -> Self {
        let mut sets = [DescriptorSetState::default(); MAX_DESCRIPTOR_SETS];

        for set in 0..MAX_DESCRIPTOR_SETS {
            sets[set].set = set as u32;
        }

        Self {
            sets,
            dirty_sets: 0,
            push_constant_data: [0; PUSH_CONSTANT_SIZE],
            push_constant_update_range: None,
        }
    }
    fn set_and_binding_exists(shader: &Arc<Shader>, mask: u32, set: u32, binding: u32) -> bool {
        let set_exists = shader.pipeline_layout().set_mask() & 1 << set == 1;
        let binding_exists = mask & 1 << binding == 1;

        set_exists && binding_exists
    }
    #[inline]
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
    #[inline]
    pub fn set_uniform_buffer(
        &mut self,
        buffer: vk::Buffer,
        start: vk::DeviceSize,
        range: vk::DeviceSize,
        set: u32,
        binding: u32,
    ) {
        self.dirty_sets |= 1 << set;

        self.sets[set as usize].set_buffer(binding, buffer, start, range);
    }

    /// Update once the whole range
    /// Data must be aligned according to GLSL std430
    /// Offset is in bytes and must be 4 byte aligned
    pub fn push_constants<T: Copy>(&mut self, data: &T, offset: usize) {
        debug_assert!(offset % 4 == 0);
        debug_assert!(std::mem::align_of::<T>() % 4 == 0);

        let byte_slice = unsafe {
            std::slice::from_raw_parts(data as *const T as *const u8, std::mem::size_of::<T>())
        };
        self.push_constant_data[offset..byte_slice.len()].copy_from_slice(byte_slice);

        self.push_constant_update_range = Some((offset, byte_slice.len()));
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
        hikari_dev::profile_function!();
        //let mut sets = crate::util::ArrayVecCopy::<vk::DescriptorSet, MAX_DESCRIPTOR_SETS>::new();
        let sets_to_update = self.dirty_sets & shader.pipeline_layout().set_mask();
        crate::util::for_each_bit(sets_to_update, |set| {
            let set_layout = &shader.pipeline_layout().set_layouts()[set as usize];

            let allocator = descriptor_pool.get(set_layout);
            let state = &self.sets[set as usize];

            //assert!(set_layout == &state.set_layout);

            let vk_set = allocator.get(state);
            let sets = [vk_set];

            unsafe {
                hikari_dev::profile_scope!("Binding descriptor set");
                device.raw().cmd_bind_descriptor_sets(
                    cmd,
                    bind_point,
                    shader.pipeline_layout().raw(),
                    set,
                    &sets,
                    &[],
                );
            }
        });
        if let Some((start, len)) = self.push_constant_update_range {
            //println!("{:?}", &self.push_constant_data[start..len]);
            hikari_dev::profile_scope!("Push Constants");
            unsafe {
                device.raw().cmd_push_constants(
                    cmd,
                    shader.pipeline_layout().raw(),
                    shader.pipeline_layout().push_constant_stage_flags(),
                    0,
                    &self.push_constant_data[start..len],
                );
            }

            self.push_constant_update_range = None;
        }

        self.dirty_sets = 0;
    }
}
pub struct CommandBufferSavedState<'a> {
    pub pipeline_lookup: &'a mut PipelineLookup,
    pub descriptor_pool: &'a mut DescriptorPool,
    pub descriptor_state: &'a mut DescriptorState,
}
pub struct CommandBuffer<'a> {
    device: &'a Arc<crate::Device>,
    saved_state: CommandBufferSavedState<'a>,
    cmd: vk::CommandBuffer,
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
            saved_state,
        }
    }
    pub fn raw(&self) -> vk::CommandBuffer {
        self.cmd
    }
    #[inline]
    pub fn begin(&self) -> VkResult<()> {
        hikari_dev::profile_function!();
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .raw()
                .begin_command_buffer(self.raw(), &begin_info)
        }
    }
    #[inline]
    pub fn end(&self) -> VkResult<()> {
        unsafe { self.device.raw().end_command_buffer(self.raw()) }
    }
    #[inline]
    pub(crate) fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32) {
        unsafe {
            self.device.raw().cmd_set_viewport(
                self.cmd,
                0,
                &[vk::Viewport {
                    x,
                    y: height - y,
                    width,
                    height: -height,
                    min_depth: 0.0,
                    max_depth: 1.0,
                }],
            );
        }
    }
    #[inline]
    pub(crate) fn set_scissor(&mut self, offset_x: i32, offset_y: i32, width: u32, height: u32) {
        unsafe {
            self.device.raw().cmd_set_scissor(
                self.cmd,
                0,
                &[vk::Rect2D {
                    offset: vk::Offset2D {
                        x: offset_x,
                        y: offset_y,
                    },
                    extent: vk::Extent2D { width, height },
                }],
            );
        }
    }
    #[inline]
    pub(crate) fn set_image(&mut self, image: &SampledImage, set: u32, binding: u32) {
        self.saved_state.descriptor_state.set_image(
            image.image_view(1).unwrap(),
            image.sampler(),
            set,
            binding,
            0,
        );
    }
    pub(crate) fn set_uniform_buffer<B: Buffer>(
        &mut self,
        buffer: &B,
        span: Range<usize>,
        set: u32,
        binding: u32,
    ) {
        let offset = buffer.offset(span.start);
        let end = buffer.offset(span.end);
        self.saved_state.descriptor_state.set_uniform_buffer(
            buffer.buffer(),
            offset,
            end - offset,
            set,
            binding,
        )
    }
    #[inline]
    pub(crate) fn begin_renderpass<'cmd>(
        &'cmd mut self,
        begin_info: RenderpassBeginInfo<'cmd>,
    ) -> RenderpassCommands<'cmd, 'a> {
        RenderpassCommands::new(self, begin_info)
    }
    pub(crate) fn reset(&mut self) -> VkResult<()> {
        hikari_dev::profile_function!();
        self.saved_state.descriptor_state.reset();

        unsafe {
            self.device
                .raw()
                .reset_command_buffer(self.raw(), vk::CommandBufferResetFlags::empty())
        }
    }
}
