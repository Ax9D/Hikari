use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

use arrayvec::ArrayVec;
use ash::prelude::VkResult;
use ash::vk;
use ash::vk::DescriptorPoolSize;
use fxhash::FxHashMap;

use crate::util::TemporaryMap;


pub const MAX_DESCRIPTOR_SETS: usize = 4;
pub const MAX_BINDINGS_PER_SET: usize = 16;

#[derive(Copy, Clone, Default)]
pub struct DescriptorSetLayout {
    vk_layout: vk::DescriptorSetLayout,
    combined_image_sampler_mask: u32,
    uniform_buffer_mask: u32,
    stage_flags: [vk::ShaderStageFlags; MAX_BINDINGS_PER_SET],
    counts: [u32; MAX_BINDINGS_PER_SET]
}

impl Hash for DescriptorSetLayout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vk_layout.hash(state);
    }
}
impl PartialEq for DescriptorSetLayout {
    fn eq(&self, other: &Self) -> bool {
        self.vk_layout == other.vk_layout
    }
}
impl Eq for DescriptorSetLayout {}

impl DescriptorSetLayout {
    pub fn new() -> DescriptorSetLayoutBuilder {
        DescriptorSetLayoutBuilder::default()
    }
    pub fn raw(&self) -> vk::DescriptorSetLayout {
        self.vk_layout
    }
    /// Get a reference to the descriptor set layout's count.
    pub const fn counts(&self) -> &[u32; MAX_BINDINGS_PER_SET] {
        &self.counts
    }

    /// Get a reference to the descriptor set layout's stages.
    pub const fn stages(&self) -> &[vk::ShaderStageFlags; MAX_BINDINGS_PER_SET] {
        &self.stage_flags
    }
    /// Get a reference to the descriptor set layout's combined image sampler mask.
    pub const fn combined_image_sampler_mask(&self) -> u32 {
        self.combined_image_sampler_mask
    }
    /// Get a reference to the descriptor set layout's uniform buffer mask.
    pub const fn uniform_buffer_mask(&self) -> u32 {
        self.uniform_buffer_mask
    }
    pub const fn all_mask(&self) -> u32 {
        self.combined_image_sampler_mask() | self.uniform_buffer_mask()
    } 
    
    pub fn binding(&self, id: u32) -> Option<(vk::DescriptorType, u32, vk::ShaderStageFlags)> {
        if self.stage_flags[id as usize].is_empty() {
            return None;
        }

        if self.combined_image_sampler_mask & 1 << id == 1 {
            return Some( (vk::DescriptorType::COMBINED_IMAGE_SAMPLER, self.counts()[id as usize], self.stages()[id as usize]) );
        }

        if self.uniform_buffer_mask & 1 << id == 1  {
            return Some( (vk::DescriptorType::UNIFORM_BUFFER, self.counts()[id as usize], self.stages()[id as usize]) );
        }

        None
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DescriptorSetLayoutBuilder {
    combined_image_sampler_mask: u32,
    uniform_buffer_mask: u32,
    stage_flags: [vk::ShaderStageFlags; MAX_BINDINGS_PER_SET],
    counts: [u32; MAX_BINDINGS_PER_SET]
}

impl DescriptorSetLayoutBuilder {
    pub fn with_binding(&mut self, id: u32, kind: vk::DescriptorType, count: u32, stage_flags: vk::ShaderStageFlags) {
        if id as usize > MAX_BINDINGS_PER_SET {
            panic!("DescriptorSets can have a maximum of {} bindings per set", MAX_BINDINGS_PER_SET);
        }
        match kind {
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER => {
                self.combined_image_sampler_mask |= 1 << id;
            },
            vk::DescriptorType::UNIFORM_BUFFER => {
                self.uniform_buffer_mask |= 1 << id;
            }
            _=> { todo!() }
        }

        self.stage_flags[id as usize] |= stage_flags;
        self.counts[id as usize] = count;
    }
    pub fn binding(&self, id: u32) -> Option<(vk::DescriptorType, u32, vk::ShaderStageFlags)> {
        if self.stage_flags[id as usize].is_empty() {
            return None;
        }

        if self.combined_image_sampler_mask & 1 << id == 1 {
            return Some( (vk::DescriptorType::COMBINED_IMAGE_SAMPLER, self.counts[id as usize], self.stage_flags[id as usize]) );
        }

        if self.uniform_buffer_mask & 1 << id == 1  {
            return Some( (vk::DescriptorType::UNIFORM_BUFFER, self.counts[id as usize], self.stage_flags[id as usize]) );
        }

        None
    }
    pub fn build(self, device: &Arc<crate::Device>) -> VkResult<DescriptorSetLayout> {
        let mut layout_cache = device.set_layout_cache();
        let layout = layout_cache.get_layout(&self)?;

        Ok(layout)
    }
}

pub(crate) struct DescriptorSetLayoutCache {
    device: ash::Device,
    layouts: FxHashMap<DescriptorSetLayoutBuilder, DescriptorSetLayout>,
}

impl DescriptorSetLayoutCache {
    pub fn new(device: &ash::Device) -> Self {
        Self {
            device: device.clone(),
            layouts: Default::default(),
        }
    }
    pub fn get_layout(
        &mut self,
        layout: &DescriptorSetLayoutBuilder,
    ) -> VkResult<DescriptorSetLayout> {
        let vk_layout = match self.layouts.get(layout) {
            Some(&layout) => layout,
            None => {
                let new_layout = self.create_set_layout(layout)?;

                self.layouts.insert(layout.clone(), new_layout);

                new_layout
            },
        };

        Ok(vk_layout)
    }
    fn create_set_layout(&mut self, layout_builder: &DescriptorSetLayoutBuilder) -> VkResult<DescriptorSetLayout> {
        let mut bindings = Vec::new();
        for binding in 0..MAX_BINDINGS_PER_SET {
            if let Some((desc_type, count, stage_flags)) = layout_builder.binding(binding as u32) {
                bindings.push(
                    *vk::DescriptorSetLayoutBinding::builder()
                    .binding(binding as u32)
                    .stage_flags(stage_flags)
                    .descriptor_count(count)
                )
            }
        }

        let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&bindings)
        .flags(vk::DescriptorSetLayoutCreateFlags::empty());

        let vk_layout = unsafe { self.device.create_descriptor_set_layout(&create_info, None) }?;

        Ok(
            DescriptorSetLayout {
                vk_layout,
                combined_image_sampler_mask: layout_builder.combined_image_sampler_mask,
                uniform_buffer_mask: layout_builder.uniform_buffer_mask,
                stage_flags: layout_builder.stage_flags,
                counts: layout_builder.counts
            }
        )
    }
}

impl Drop for DescriptorSetLayoutCache {
    fn drop(&mut self) {
        for (_, layout) in self.layouts.drain() {
            unsafe {
                self.device
                    .destroy_descriptor_set_layout(layout.vk_layout, None);
            }
        }

        log::debug!("Dropped DescriptorSetLayoutCache");
    }
}

pub const MAX_IMAGE_COUNT: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct ImageState {
    images: [(vk::ImageView, vk::Sampler); MAX_IMAGE_COUNT],
    image_update_mask: u32
}
impl Hash for ImageState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        crate::util::for_each_bit(self.image_update_mask, 0..MAX_IMAGE_COUNT, |image_ix| {
            self.images[image_ix as usize].hash(state);
        });
    }
}
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
struct BufferState {
    buffer: vk::Buffer,
    offset: vk::DeviceSize,
    range: vk::DeviceSize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct BindingState {
    image_state: ImageState,
    buffer_state: BufferState
}
impl BindingState {

    pub fn set_image(&mut self, ix: usize, image_view: vk::ImageView, sampler: vk::Sampler) {
        self.image_state.images[ix] = (image_view, sampler);
        self.image_state.image_update_mask &= 1 << ix;
    }
    pub fn set_buffer(&mut self, buffer: vk::Buffer, offset: vk::DeviceSize, range: vk::DeviceSize) {
        self.buffer_state.buffer = buffer;
        self.buffer_state.offset = offset;
        self.buffer_state.range = range;
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct DescriptorSetState {
    pub set: u32,
    pub set_layout: DescriptorSetLayout,
    bindings: [BindingState; MAX_BINDINGS_PER_SET]
}
impl DescriptorSetState {
    pub fn new(set: u32, set_layout: DescriptorSetLayout) -> Self {
        Self {
            set,
            set_layout,
            bindings: [BindingState::default(); MAX_BINDINGS_PER_SET]
        }
    }
    pub fn set_image(&mut self, binding: u32, ix: usize, image_view: vk::ImageView, sampler: vk::Sampler) {
            self.bindings[binding as usize].set_image(ix, image_view, sampler);
    }
    pub fn set_buffer(&mut self, binding: u32, buffer: vk::Buffer, offset: vk::DeviceSize, range: vk::DeviceSize) {
        self.bindings[binding as usize].set_buffer(buffer, offset, range);
    } 
}
impl Hash for DescriptorSetState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        crate::util::for_each_bit(self.set_layout.combined_image_sampler_mask(), 0..MAX_BINDINGS_PER_SET, |binding| {
            self.bindings[self.set as usize].image_state.hash(state);
            
        });

        crate::util::for_each_bit(self.set_layout.uniform_buffer_mask(), 0..MAX_BINDINGS_PER_SET, |binding| {
            self.bindings[self.set as usize].buffer_state.hash(state);
        });
    }
}
pub const MAX_SETS_PER_POOL: usize = 1000;
pub struct DescriptorSetAllocator {
    device: Arc<crate::Device>,
    set_layout: DescriptorSetLayout,
    max_sets: u32,
    temp_map: TemporaryMap<DescriptorSetState, vk::DescriptorSet, 4>,
    resuable_sets: Vec<vk::DescriptorSet>,
    
    pools: Vec<vk::DescriptorPool>,
    pool_sizes: Vec<vk::DescriptorPoolSize>
}
impl DescriptorSetAllocator {
    pub fn new(
        device: &Arc<crate::Device>,
        set_layout: DescriptorSetLayout,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(
            Self {
            device: device.clone(),
            set_layout,
            max_sets: MAX_SETS_PER_POOL as u32,
            temp_map: TemporaryMap::new(),
            resuable_sets: Vec::new(),
            pools: Vec::new(),
            pool_sizes: Self::get_pool_sizes(MAX_SETS_PER_POOL, set_layout)
        }
    )
    }

    fn get_pool_sizes(count_per_binding: usize, set_layout: DescriptorSetLayout) -> Vec<vk::DescriptorPoolSize> {
        let mut pool_sizes_map: HashMap<vk::DescriptorType, usize> = HashMap::new();

        for binding in 0..MAX_BINDINGS_PER_SET {
            if let Some((desc_type, count, _)) = set_layout.binding(binding as u32) {
                *pool_sizes_map.entry(desc_type).or_default() += count as usize;
            }
        }

            pool_sizes_map
            .iter()
            .map(|(ty, n)| {
                *DescriptorPoolSize::builder()
                    .ty(*ty)
                    .descriptor_count(( count_per_binding * (*n) ) as u32)
            })
            .collect()
    }
    fn create_pool(&self) -> VkResult<vk::DescriptorPool> {
        let create_info = vk::DescriptorPoolCreateInfo::builder()
        .max_sets(self.max_sets)
        .pool_sizes(&self.pool_sizes)
        .flags(vk::DescriptorPoolCreateFlags::empty());

        unsafe {
            self.device.raw().create_descriptor_pool(&create_info, None)
        }
    }
    fn current_pool(&self) -> vk::DescriptorPool {
        self.pools[self.pools.len()]
    }
    fn allocate(&mut self, vk_set_layout: vk::DescriptorSetLayout) -> VkResult<vk::DescriptorSet> {
        unsafe {
            let layouts = [vk_set_layout];
            let create_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.current_pool())
            .set_layouts(&layouts);

            let result = self.device.raw().allocate_descriptor_sets(&create_info);

            match result {
                Ok(set) => {
                    Ok(set[0])
                },
                Err(err_kind) => {
                    match err_kind {
                        vk::Result::ERROR_OUT_OF_POOL_MEMORY | vk::Result::ERROR_FRAGMENTED_POOL => {
                            self.pools.push(self.create_pool()?);
                            self.allocate(vk_set_layout)
                        },
                        _=> panic!("Descriptor Set allocation failed")
                    }
                },
            }
        }
    }
    fn update_set(&self, set: vk::DescriptorSet, state: &DescriptorSetState) {
        const MAX_WRITES: usize = MAX_BINDINGS_PER_SET * MAX_IMAGE_COUNT;

        let mut writes = ArrayVec::<vk::WriteDescriptorSet, MAX_WRITES>::new();

        crate::util::for_each_bit(state.set_layout.combined_image_sampler_mask(), 0..MAX_BINDINGS_PER_SET, |binding| {
            let image_state = &state.bindings[binding as usize].image_state;

            crate::util::for_each_bit(image_state.image_update_mask, 0..MAX_IMAGE_COUNT, |image_ix| {

                let (image_view, sampler) = image_state.images[image_ix as usize];

                let write = *vk::WriteDescriptorSet::builder()
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .dst_set(set)
                .dst_binding(binding)
                .dst_array_element(image_ix)
                .image_info(
                    &[vk::DescriptorImageInfo {
                    image_view,
                    sampler,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
                }]);

                writes.push(write);

            });

        }); 

        crate::util::for_each_bit(state.set_layout.uniform_buffer_mask(), 0..MAX_BINDINGS_PER_SET, |binding| {
            let buffer_state = &state.bindings[binding as usize].buffer_state;

            if buffer_state.buffer!=vk::Buffer::null() {
                let write  = *vk::WriteDescriptorSet::builder()
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .dst_set(set)
                        .dst_binding(binding)
                        .dst_array_element(0)
                        .buffer_info(
                            &[vk::DescriptorBufferInfo {
                            buffer: buffer_state.buffer,
                            offset: buffer_state.offset,
                            range: buffer_state.range
                        }]);

                writes.push(write);
            }
        });


        unsafe {
            self.device.raw().update_descriptor_sets(&writes, &[]);
        }
        
    }
    pub fn get(&mut self, state: &DescriptorSetState) -> vk::DescriptorSet {
        if let Some(reusable_set) = self.resuable_sets.pop() {
            self.update_set(reusable_set, state);

            reusable_set
        } else {
            match self.temp_map.get(state) {
                Some(&set) => set,
                None => {
                    let allocated = self.allocate(self.set_layout.raw()).unwrap();
                    self.temp_map.insert(state.clone(), allocated);

                    self.update_set(allocated, state);

                    allocated
                },
            }
        }
    }
    fn new_frame(&mut self) {
        let reusable_sets = &mut self.resuable_sets;
        self.temp_map.new_frame(|set| reusable_sets.push(set));
    }

    // fn default_descriptor_pool(
    //     device: ash::Device,
    //     count: u32,
    //     flags: vk::DescriptorPoolCreateFlags,
    // ) -> VkResult<vk::DescriptorPool> {
    //     use vk::DescriptorType;

    //     //From https://vkguide.dev/docs/extra-chapter/abstracting_descriptors/
    //     let pool_sizes = [
    //         (DescriptorType::SAMPLER, 0.5),
    //         (DescriptorType::COMBINED_IMAGE_SAMPLER, 4.0),
    //         (DescriptorType::SAMPLED_IMAGE, 4.0),
    //         (DescriptorType::STORAGE_BUFFER, 1.0),
    //         (DescriptorType::UNIFORM_TEXEL_BUFFER, 1.0),
    //         (DescriptorType::STORAGE_TEXEL_BUFFER, 1.0),
    //         (DescriptorType::UNIFORM_BUFFER, 2.0),
    //         (DescriptorType::STORAGE_BUFFER, 2.0),
    //         (DescriptorType::UNIFORM_BUFFER_DYNAMIC, 1.0),
    //         (DescriptorType::STORAGE_BUFFER_DYNAMIC, 1.0),
    //         (DescriptorType::INPUT_ATTACHMENT, 0.5),
    //     ];

    //     let pool_sizes: Vec<_> = pool_sizes
    //         .iter()
    //         .map(|(ty, n)| {
    //             DescriptorPoolSize::builder()
    //                 .ty(*ty)
    //                 .descriptor_count((count as f32 * (*n)) as u32)
    //                 .build()
    //         })
    //         .collect();
    //     let create_info = vk::DescriptorPoolCreateInfo::builder()
    //         .pool_sizes(&pool_sizes)
    //         .max_sets(count)
    //         .flags(flags);

    //     unsafe { device.create_descriptor_pool(&create_info, None) }
    // }
    //REFACTOR THIS SHIT INTO A GRACEFUL DROP SOMEHOW
    // pub unsafe fn free(&mut self) {
    //     unsafe {
    //         self.device.reset_descriptor_pool(self.pool, vk::DescriptorPoolResetFlags::empty()).unwrap();
    //         self.device.destroy_descriptor_pool(self.pool, None);
    //     }
    //     log::debug!("Dropped DescriptorSetAllocator");
    // }
}

impl Drop for DescriptorSetAllocator {
    fn drop(&mut self) {
        for pool in self.pools.drain(..) {
            unsafe { self.device.raw().destroy_descriptor_pool(pool, None); }
        }
        log::debug!("Dropped DescriptorSetAllocator");
    }
}

pub struct DescriptorPool {
    device: Arc<crate::Device>,
    set_allocators: FxHashMap<vk::DescriptorSetLayout, DescriptorSetAllocator>
}
impl DescriptorPool {
    pub fn new(device: &Arc<crate::Device>) -> Self {
        Self {
            device: device.clone(),
            set_allocators: FxHashMap::default()
        }
    }
    pub fn get(&mut self, set_layout: &DescriptorSetLayout) -> &mut DescriptorSetAllocator {
        match self.set_allocators.entry(set_layout.raw()) {
            Entry::Occupied(allocator) => allocator.into_mut(),
            Entry::Vacant(vacant) => {
                vacant.insert(DescriptorSetAllocator::new(&self.device, set_layout.clone()).unwrap())
            },
        }
    }
    pub fn new_frame(&mut self) {
        for allocator in self.set_allocators.values_mut() {
            allocator.new_frame();
        } 
    }
}
