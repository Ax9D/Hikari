use std::{ops::Range, sync::Arc};

use ash::{prelude::VkResult, vk};

use vk_sync_fork::{AccessType, ImageBarrier};

use crate::{
    barrier,
    buffer::Buffer,
    descriptor::{DescriptorPool, DescriptorSetState, MAX_DESCRIPTOR_SETS, BINDLESS_SET_ID},
    image::SampledImage,
    util::CacheMap,
    Shader,
};

use self::render::PipelineStateVector;

pub mod compute;
pub mod render;

pub use render::PassRecordInfo;
pub use render::RenderpassBeginInfo;
pub use render::RenderpassCommands;

const PUSH_CONSTANT_SIZE: usize = 128;
const PUSH_CONSTANT_SIZEU32: usize = PUSH_CONSTANT_SIZE / std::mem::size_of::<u32>();

/// Updates shader according to vulkan pipeline compatibility rules
/// Return type indicates if the pipeline is dirty
pub fn update_shader(current: &mut Option<Arc<crate::Shader>>, new: &Arc<crate::Shader>, dirty_sets: &mut u32) -> bool {
    if let Some(old_shader) = current.replace(new.clone()) {
        if old_shader.hash == new.hash {
            return false;
        }
        for set in 0..MAX_DESCRIPTOR_SETS {
            let old_layout = &old_shader.pipeline_layout().set_layouts()[set];
            let new_layout = &old_shader.pipeline_layout().set_layouts()[set];

            if old_layout != new_layout {
                *dirty_sets |= !((1 << set) - 1);
            }
        }
    }

    true
}

#[derive(Clone, Copy)]
pub struct DescriptorState {
    bindless_set: vk::DescriptorSet,
    sets: [DescriptorSetState; MAX_DESCRIPTOR_SETS],
    push_constant_data: [u8; PUSH_CONSTANT_SIZE],
    dirty_sets: u32,
    push_constant_update_range: Option<(usize, usize)>,
}

impl Default for DescriptorState {
    fn default() -> Self {
        Self {
            bindless_set: vk::DescriptorSet::null(),
            sets: Default::default(),
            push_constant_data: [0; PUSH_CONSTANT_SIZE],
            dirty_sets: 0,
            push_constant_update_range: None,
        }
    }
}

impl DescriptorState {
    pub fn new() -> Self {
        Self::default()
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
        debug_assert!(set != BINDLESS_SET_ID);
        self.dirty_sets |= 1 << set;

        self.sets[set as usize].set_image(binding, ix, image_view, sampler);
    }
    #[inline]
    pub fn set_buffer(
        &mut self,
        buffer: vk::Buffer,
        start: vk::DeviceSize,
        range: vk::DeviceSize,
        set: u32,
        binding: u32,
    ) {
        debug_assert!(set != BINDLESS_SET_ID);
        self.dirty_sets |= 1 << set;

        self.sets[set as usize].set_buffer(binding, buffer, start, range);
    }
    #[inline]
    pub fn set_bindless(&mut self, set: vk::DescriptorSet) {
        self.bindless_set = set;

        self.dirty_sets |= 1 << BINDLESS_SET_ID;
    }

    /// Update once the whole range
    /// Data must be aligned according to GLSL std430
    /// Offset is in bytes and must be 4 byte aligned
    pub fn push_constants<T: Copy>(&mut self, data: &T, offset: usize) {
        hikari_dev::profile_scope!("Push Constant CPU Update");
        debug_assert!(offset % 4 == 0);
        debug_assert!(std::mem::align_of::<T>() % 4 == 0);

        let byte_slice = unsafe {
            std::slice::from_raw_parts(data as *const T as *const u8, std::mem::size_of::<T>())
        };
        self.push_constant_data[offset..offset + byte_slice.len()].copy_from_slice(byte_slice);

        self.push_constant_update_range = Some((offset, byte_slice.len()));
    }
    pub fn reset(&mut self) {
        for set in &mut self.sets {
            set.reset();
        }
        self.bindless_set = vk::DescriptorSet::null();
    }
    #[inline]
    pub fn set_all_dirty(&mut self) {
        self.dirty_sets = !(!0 << MAX_DESCRIPTOR_SETS);
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
        let pipeline_layout = shader.pipeline_layout();
        let set_layouts = pipeline_layout.set_layouts();
        let pipeline_sets = shader.pipeline_layout().set_mask();
        let mut sets_to_update = self.dirty_sets & pipeline_sets;

        if (sets_to_update >> BINDLESS_SET_ID) & 1 == 1 {
            let sets = [self.bindless_set];
            unsafe {
                hikari_dev::profile_scope!("Binding bindless descriptor set");
                device.raw().cmd_bind_descriptor_sets(
                    cmd,
                    bind_point,
                    shader.pipeline_layout().raw(),
                    0,
                    &sets,
                    &[],
                );
            }

            sets_to_update ^= 1 << BINDLESS_SET_ID;
        }
        
        crate::util::for_each_bit(sets_to_update, |set| {
            let set_layout = &set_layouts[set as usize];

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
                    start as u32,
                    &self.push_constant_data[start..start + len],
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
    cmd: vk::CommandBuffer,
    saved_state: CommandBufferSavedState<'a>,
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
    #[inline]
    pub fn raw(&self) -> vk::CommandBuffer {
        self.cmd
    }
    #[inline]
    pub fn begin(&self) -> VkResult<()> {
        hikari_dev::profile_function!();
        let begin_info = vk::CommandBufferBeginInfo::default()
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
        self.set_image_mip(image, 0, set, binding)
    }
    #[inline]
    pub(crate) fn set_image_mip(
        &mut self,
        image: &SampledImage,
        mip_level: u32,
        set: u32,
        binding: u32,
    ) {
        self.set_image_view_and_sampler(
            image.shader_resource_view(mip_level).unwrap(),
            image.sampler(),
            set,
            binding,
            0,
        )
    }
    #[inline]
    pub(crate) fn set_image_view_and_sampler(
        &mut self,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
        set: u32,
        binding: u32,
        index: usize,
    ) {
        self.saved_state
            .descriptor_state
            .set_image(image_view, sampler, set, binding, index);
    }
    #[inline]
    pub(crate) fn set_image_array(
        &mut self,
        image: &SampledImage,
        set: u32,
        binding: u32,
        index: usize,
    ) {
        self.set_image_mip_array(image, 1, set, binding, index)
    }
    #[inline]
    pub(crate) fn set_image_mip_array(
        &mut self,
        image: &SampledImage,
        mip_level: u32,
        set: u32,
        binding: u32,
        index: usize,
    ) {
        self.saved_state.descriptor_state.set_image(
            image.shader_resource_view(mip_level).unwrap(),
            image.sampler(),
            set,
            binding,
            index,
        );
    }
    pub(crate) fn apply_image_barrier(
        &mut self,
        image: &SampledImage,
        previous_accesses: &[AccessType],
        next_accesses: &[AccessType],
        previous_layout: crate::vk_sync::ImageLayout,
        next_layout: crate::vk_sync::ImageLayout,
        range: vk::ImageSubresourceRange,
    ) {
        let barrier = ImageBarrier {
            previous_accesses,
            next_accesses,
            previous_layout,
            next_layout,
            discard_contents: false,
            src_queue_family_index: self.device.unified_queue_ix,
            dst_queue_family_index: self.device.unified_queue_ix,
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
        ) = vk_sync_fork::get_image_memory_barrier(&barrier);

        let barrier = [vk::ImageMemoryBarrier2KHR::default()
            .image(image)
            .subresource_range(range)
            .src_access_mask(barrier::to_sync2_access_flags(src_access_mask))
            .dst_access_mask(barrier::to_sync2_access_flags(dst_access_mask))
            .src_stage_mask(barrier::to_sync2_stage_flags(src_stage_mask))
            .dst_stage_mask(barrier::to_sync2_stage_flags(dst_stage_mask))
            .old_layout(old_layout)
            .new_layout(new_layout)];

        let dependency_info = vk::DependencyInfoKHR::default()
            .image_memory_barriers(&barrier)
            .dependency_flags(vk::DependencyFlags::BY_REGION);

        unsafe {
            self.device
                .extensions()
                .synchronization2
                .cmd_pipeline_barrier2(self.cmd, &dependency_info);
        }
    }
    pub(crate) fn set_buffer<B: Buffer>(
        &mut self,
        buffer: &B,
        span: Range<usize>,
        set: u32,
        binding: u32,
    ) {
        let offset = buffer.offset(span.start);
        let end = buffer.offset(span.end);
        self.saved_state.descriptor_state.set_buffer(
            buffer.buffer(),
            offset,
            end - offset,
            set,
            binding,
        )
    }
    pub(crate) fn set_bindless(&mut self, set: vk::DescriptorSet) {
        self.saved_state.descriptor_state.set_bindless(set)
    }
    #[inline]
    pub(crate) fn begin_renderpass<'cmd>(
        &'cmd mut self,
        begin_info: RenderpassBeginInfo<'cmd>,
    ) -> RenderpassCommands<'cmd, 'a> {
        RenderpassCommands::new(self, begin_info)
    }
    pub(crate) fn begin_debug_region<'cmd>(
        &'cmd mut self,
        name: impl AsRef<str>,
        color: hikari_math::Vec4,
    ) {
        if let Some(debug_utils) = &self.device.extensions().debug_utils {
            let name = name.as_ref();
            let name_cstring =
                std::ffi::CString::new(name).expect("Debug Label is not a valid CString");

            let label = vk::DebugUtilsLabelEXT::default()
                .label_name(&name_cstring)
                .color(color.to_array());

            unsafe {
                debug_utils.cmd_begin_debug_utils_label(self.cmd, &label);
            }
        }
    }
    pub(crate) fn end_debug_region<'cmd>(&'cmd mut self) {
        if let Some(debug_utils) = &self.device.extensions().debug_utils {
            unsafe {
                debug_utils.cmd_end_debug_utils_label(self.cmd);
            }
        }
    }
    pub fn copy_image(
        &self,
        src: &SampledImage,
        src_layout: vk::ImageLayout,
        dst: &SampledImage,
        dst_layout: vk::ImageLayout,
        copy_info: &[vk::ImageCopy],
    ) {
        unsafe {
            self.device.raw().cmd_copy_image(
                self.cmd,
                src.image(),
                src_layout,
                dst.image(),
                dst_layout,
                copy_info,
            );
        }
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

pub struct PipelineLookup {
    device: Arc<crate::Device>,
    vk_pipeline_cache: vk::PipelineCache,
    graphics_pipelines: CacheMap<PipelineStateVector, vk::Pipeline>,
    compute_pipelines: CacheMap<Arc<Shader>, vk::Pipeline>,
}

impl PipelineLookup {
    pub fn new(
        device: &Arc<crate::Device>,
        capacity: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            device: device.clone(),
            vk_pipeline_cache: device.pipeline_cache(),
            graphics_pipelines: CacheMap::new(capacity / 2),
            compute_pipelines: CacheMap::new(capacity / 2),
        })
    }
    fn destroy_pipeline(device: &Arc<crate::Device>, vk_pipeline: vk::Pipeline) {
        unsafe {
            device.raw().destroy_pipeline(vk_pipeline, None);
            log::debug!("Destroyed pipeline: {:?}", vk_pipeline);
        }
    }
    pub fn get_vk_graphics_pipeline(
        &mut self,
        pipeline_state_vector: &PipelineStateVector,
        renderpass: vk::RenderPass,
        n_color_attachments: usize,
    ) -> anyhow::Result<vk::Pipeline> {
        let device = &self.device;
        let pipeline = self
        .graphics_pipelines
        .get::<anyhow::Error>(pipeline_state_vector, |psv| unsafe {
                let Some(shader) = &psv.shader else {
                     return Err(anyhow::anyhow!("Shader must not be None")) 
                };

                let pipeline = psv.pipeline_state.create_pipeline(
                    device,
                    &shader,
                    renderpass,
                    n_color_attachments,
                );

                Ok(pipeline)
            })?;

        Ok(*pipeline)
    }
    pub fn get_vk_compute_pipeline(
        &mut self,
        shader: &Arc<crate::Shader>,
    ) -> VkResult<vk::Pipeline> {
        let pipeline = self.compute_pipelines.get(shader, |shader| unsafe {
            let create_info = vk::ComputePipelineCreateInfo::default()
                .stage(shader.vk_stages()[0])
                .layout(shader.pipeline_layout().raw());
            unsafe {
                let pipelines = self
                    .device
                    .raw()
                    .create_compute_pipelines(self.vk_pipeline_cache, &[create_info], None)
                    .unwrap();
                Ok(pipelines[0])
            }
        })?;

        Ok(*pipeline)
    }

    //Call once per frame
    pub fn new_frame(&mut self) {
        let device = &self.device;
        self.graphics_pipelines
            .unused()
            .drain(..)
            .for_each(|pipeline| Self::destroy_pipeline(device, pipeline));
        self.compute_pipelines
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
