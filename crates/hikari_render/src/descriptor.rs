use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

use ash::prelude::VkResult;
use ash::vk;
use ash::vk::DescriptorPoolSize;

use crate::util::ArrayVecCopy;
use crate::util::TemporaryMap;

pub const MAX_DESCRIPTOR_SETS: usize = 4;
pub const MAX_BINDINGS_PER_SET: usize = 16;
pub const MAX_COUNTS_PER_BINDING: usize = 5;
pub const MAX_BINDLESS_COUNT: usize = 4096;
pub const BINDLESS_SET_ID: u32 = 0;

#[derive(Copy, Clone, Default)]
pub struct DescriptorSetLayout {
    vk_layout: vk::DescriptorSetLayout,
    create_flags: vk::DescriptorSetLayoutCreateFlags,
    combined_image_sampler_mask: u32,
    storage_image_mask: u32,
    uniform_buffer_mask: u32,
    storage_buffer_mask: u32,
    stage_flags: [vk::ShaderStageFlags; MAX_BINDINGS_PER_SET],
    counts: [u32; MAX_BINDINGS_PER_SET],
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
    pub fn builder() -> DescriptorSetLayoutBuilder {
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
    /// Get a reference to the descriptor set layout's storage image mask.
    pub const fn storage_image_mask(&self) -> u32 {
        self.storage_image_mask
    }
    /// Get a reference to the descriptor set layout's uniform buffer mask.
    pub const fn uniform_buffer_mask(&self) -> u32 {
        self.uniform_buffer_mask
    }
    // Get a reference to the descriptor set layout's storage buffer mask.
    pub const fn storage_buffer_mask(&self) -> u32 {
        self.storage_buffer_mask
    }
    pub const fn all_mask(&self) -> u32 {
        self.combined_image_sampler_mask()
            | self.storage_image_mask()
            | self.uniform_buffer_mask()
            | self.storage_buffer_mask()
    }

    pub fn binding(&self, id: u32) -> Option<(vk::DescriptorType, u32, vk::ShaderStageFlags)> {
        if self.stage_flags[id as usize].is_empty() {
            return None;
        }

        if self.combined_image_sampler_mask >> id & 1 == 1 {
            return Some((
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                self.counts()[id as usize],
                self.stages()[id as usize],
            ));
        }

        if self.storage_image_mask >> id & 1 == 1 {
            return Some((
                vk::DescriptorType::STORAGE_IMAGE,
                self.counts()[id as usize],
                self.stages()[id as usize],
            ));
        }

        if self.uniform_buffer_mask >> id & 1 == 1 {
            return Some((
                vk::DescriptorType::UNIFORM_BUFFER,
                self.counts()[id as usize],
                self.stages()[id as usize],
            ));
        }

        if self.storage_buffer_mask >> id & 1 == 1 {
            return Some((
                vk::DescriptorType::STORAGE_BUFFER,
                self.counts()[id as usize],
                self.stages()[id as usize],
            ));
        }

        None
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DescriptorSetLayoutBuilder {
    combined_image_sampler_mask: u32,
    storage_image_mask: u32,
    uniform_buffer_mask: u32,
    storage_buffer_mask: u32,
    create_flags: vk::DescriptorSetLayoutCreateFlags,
    binding_flags: [vk::DescriptorBindingFlags; MAX_BINDINGS_PER_SET],
    stage_flags: [vk::ShaderStageFlags; MAX_BINDINGS_PER_SET],
    counts: [u32; MAX_BINDINGS_PER_SET],
}

impl DescriptorSetLayoutBuilder {
    pub fn create_flags(&mut self, create_flags: vk::DescriptorSetLayoutCreateFlags) -> &mut Self {
        self.create_flags = create_flags;

        self
    }
    pub fn with_binding(
        &mut self,
        id: u32,
        kind: vk::DescriptorType,
        count: u32,
        stage_flags: vk::ShaderStageFlags,
        binding_flags: vk::DescriptorBindingFlags,
    ) -> &mut Self {
        if id as usize > MAX_BINDINGS_PER_SET {
            panic!(
                "DescriptorSets can have a maximum of {} bindings per set",
                MAX_BINDINGS_PER_SET
            );
        }
        match kind {
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER => {
                self.combined_image_sampler_mask |= 1 << id;
            }
            vk::DescriptorType::STORAGE_IMAGE => {
                self.storage_image_mask |= 1 << id;
            }
            vk::DescriptorType::UNIFORM_BUFFER => {
                self.uniform_buffer_mask |= 1 << id;
            }
            vk::DescriptorType::STORAGE_BUFFER => {
                self.storage_buffer_mask |= 1 << id;
            }
            _ => todo!(),
        }
        self.stage_flags[id as usize] |= stage_flags;
        self.binding_flags[id as usize] |= binding_flags;
        self.counts[id as usize] = count;

        self
    }
    pub fn binding(&self, id: u32) -> Option<(vk::DescriptorType, u32, vk::ShaderStageFlags, vk::DescriptorBindingFlags)> {
        if self.stage_flags[id as usize].is_empty() {
            return None;
        }

        if (self.combined_image_sampler_mask >> id) & 1 == 1 {
            return Some((
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                self.counts[id as usize],
                self.stage_flags[id as usize],
                self.binding_flags[id as usize]
            ));
        }

        if (self.storage_image_mask >> id) & 1 == 1 {
            return Some((
                vk::DescriptorType::STORAGE_IMAGE,
                self.counts[id as usize],
                self.stage_flags[id as usize],
                self.binding_flags[id as usize]
            ));
        }

        if (self.uniform_buffer_mask >> id) & 1 == 1 {
            return Some((
                vk::DescriptorType::UNIFORM_BUFFER,
                self.counts[id as usize],
                self.stage_flags[id as usize],
                self.binding_flags[id as usize]
            ));
        }

        if (self.storage_buffer_mask >> id) & 1 == 1 {
            return Some((
                vk::DescriptorType::STORAGE_BUFFER,
                self.counts[id as usize],
                self.stage_flags[id as usize],
                self.binding_flags[id as usize]
            ));
        }

        None
    }
    pub fn build(self, device: &crate::Device) -> VkResult<DescriptorSetLayout> {
        let mut layout_cache = device.cache().set_layout();
        let layout = layout_cache.get_layout(&self)?;

        Ok(layout)
    }
}

pub(crate) struct DescriptorSetLayoutCache {
    device: ash::Device,
    layouts: HashMap<DescriptorSetLayoutBuilder, DescriptorSetLayout, crate::util::BuildHasher>,
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

                self.layouts.insert(*layout, new_layout);

                new_layout
            }
        };

        Ok(vk_layout)
    }
    fn create_set_layout(
        &mut self,
        layout_builder: &DescriptorSetLayoutBuilder,
    ) -> VkResult<DescriptorSetLayout> {
        let mut bindings = Vec::new();
        let mut binding_flags_vec = Vec::new();
        for binding in 0..MAX_BINDINGS_PER_SET {
            if let Some((desc_type, count, stage_flags, binding_flags)) = layout_builder.binding(binding as u32) {
                bindings.push(
                    *vk::DescriptorSetLayoutBinding::builder()
                        .binding(binding as u32)
                        .stage_flags(stage_flags)
                        .descriptor_type(desc_type)
                        .descriptor_count(count),
                );

                binding_flags_vec.push(
                    binding_flags
                );
            }
        }

        let mut binding_flags = *vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder()
        .binding_flags(&binding_flags_vec);

        let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&bindings)
            .flags(layout_builder.create_flags)
            .push_next(&mut binding_flags);

        let vk_layout = unsafe { self.device.create_descriptor_set_layout(&create_info, None) }?;

        Ok(DescriptorSetLayout {
            vk_layout,
            create_flags: layout_builder.create_flags,
            combined_image_sampler_mask: layout_builder.combined_image_sampler_mask,
            storage_image_mask: layout_builder.storage_image_mask,
            uniform_buffer_mask: layout_builder.uniform_buffer_mask,
            storage_buffer_mask: layout_builder.storage_buffer_mask,
            stage_flags: layout_builder.stage_flags,
            counts: layout_builder.counts,
        })
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

#[derive(Debug, Clone, Copy, Eq, Default)]
struct ImageState {
    images: [(vk::ImageView, vk::Sampler); MAX_COUNTS_PER_BINDING],
    image_update_mask: u32,
}
impl Hash for ImageState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.image_update_mask.hash(state);

        crate::util::for_each_bit_in_range(
            self.image_update_mask,
            0..MAX_COUNTS_PER_BINDING,
            |image_ix| {
                self.images[image_ix as usize].hash(state);
            },
        );
    }
}
impl PartialEq for ImageState {
    fn eq(&self, other: &Self) -> bool {
        let mut same = true;

        if self.image_update_mask != other.image_update_mask {
            return false;
        }

        crate::util::for_each_bit_in_range(
            self.image_update_mask,
            0..MAX_COUNTS_PER_BINDING,
            |image_ix| {
                same = self.images[image_ix as usize] == other.images[image_ix as usize];
            },
        );

        same
    }
}
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
struct BufferState {
    buffer: vk::Buffer,
    offset: vk::DeviceSize,
    range: vk::DeviceSize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct BindingState {
    image_state: ImageState,
    buffer_state: BufferState,
}
impl BindingState {
    pub fn set_image(&mut self, ix: usize, image_view: vk::ImageView, sampler: vk::Sampler) {
        self.image_state.images[ix] = (image_view, sampler);
        self.image_state.image_update_mask |= 1 << ix;
    }
    pub fn set_buffer(&mut self, buffer: vk::Buffer, start: vk::DeviceSize, range: vk::DeviceSize) {
        self.buffer_state.buffer = buffer;
        self.buffer_state.offset = start;
        self.buffer_state.range = range;
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct DescriptorSetState {
    pub set: u32,
    //pub set_layout: DescriptorSetLayout,
    pub bindings: [BindingState; MAX_BINDINGS_PER_SET],
}
impl DescriptorSetState {
    pub fn new(set: u32, set_layout: DescriptorSetLayout) -> Self {
        Self {
            set,
            //set_layout,
            bindings: [BindingState::default(); MAX_BINDINGS_PER_SET],
        }
    }
    #[inline]
    pub fn set_image(
        &mut self,
        binding: u32,
        ix: usize,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
    ) {
        self.bindings[binding as usize].set_image(ix, image_view, sampler);
    }
    #[inline]
    pub fn set_buffer(
        &mut self,
        binding: u32,
        buffer: vk::Buffer,
        start: vk::DeviceSize,
        range: vk::DeviceSize,
    ) {
        self.bindings[binding as usize].set_buffer(buffer, start, range);
    }
    fn hash(&self, set_layout: &DescriptorSetLayout) -> u64 {
        hikari_dev::profile_function!();

        let mut state = crate::util::hasher();

        crate::util::for_each_bit_in_range(
            set_layout.combined_image_sampler_mask() | set_layout.storage_image_mask(),
            0..MAX_BINDINGS_PER_SET,
            |binding| {
                self.bindings[binding as usize].image_state.hash(&mut state);
            },
        );

        crate::util::for_each_bit_in_range(
            set_layout.uniform_buffer_mask() | set_layout.storage_buffer_mask(),
            0..MAX_BINDINGS_PER_SET,
            |binding| {
                self.bindings[binding as usize]
                    .buffer_state
                    .hash(&mut state);
            },
        );

        state.finish()
    }
    pub fn reset(&mut self) {
        hikari_dev::profile_function!();
        self.bindings = [BindingState::default(); MAX_BINDINGS_PER_SET];
    }
}

pub const MAX_SETS_PER_POOL: usize = 1000;

pub(crate) struct RawDescriptorSetAllocator {
    max_sets: u32,
    temp_map: TemporaryMap<u64, vk::DescriptorSet, 4, hikari_utils::hash::BuildNoHashHasher<u64>>,
    resuable_sets: Vec<vk::DescriptorSet>,

    pools: Vec<vk::DescriptorPool>,
    pool_sizes: Vec<vk::DescriptorPoolSize>,
}

impl RawDescriptorSetAllocator {
    pub fn new(
        device: &crate::Device,
        set_layout: &DescriptorSetLayout,
    ) -> VkResult<Self> {
        Self::with_capacity(device, &set_layout, MAX_SETS_PER_POOL)
    }
    pub fn with_capacity(device: &crate::Device, set_layout: &DescriptorSetLayout, max_sets_per_pool: usize) -> VkResult<Self> {
        let mut allocator = Self {
            max_sets: MAX_SETS_PER_POOL as u32,
            temp_map: TemporaryMap::with_hasher(hikari_utils::hash::BuildNoHashHasher::default()),
            resuable_sets: Vec::new(),
            pools: Vec::new(),
            pool_sizes: Self::get_pool_sizes(max_sets_per_pool, set_layout),
        };

        allocator.pools.push(allocator.create_pool(device, set_layout)?);

        Ok(allocator)
    }
    fn get_pool_sizes(
        count_per_binding: usize,
        set_layout: &DescriptorSetLayout,
    ) -> Vec<vk::DescriptorPoolSize> {
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
                    .descriptor_count((count_per_binding * (*n)) as u32)
            })
            .collect()
    }
    fn create_pool(&self, device: &crate::Device, set_layout: &DescriptorSetLayout) -> VkResult<vk::DescriptorPool> {
        let mut pool_create_flags = vk::DescriptorPoolCreateFlags::empty();
        
        if set_layout.create_flags.contains(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL) {
            pool_create_flags |= vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND;
        }

        let create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(self.max_sets)
            .pool_sizes(&self.pool_sizes)
            .flags(pool_create_flags);

        unsafe { device.raw().create_descriptor_pool(&create_info, None) }
    }
    fn current_pool(&self) -> vk::DescriptorPool {
        self.pools[self.pools.len() - 1]
    }
    pub fn allocate(&mut self, device: &crate::Device, layout: &DescriptorSetLayout) -> VkResult<vk::DescriptorSet> {
        hikari_dev::profile_function!();
        unsafe {
            let layouts = [layout.raw()];
            let create_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(self.current_pool())
                .set_layouts(&layouts);

            let result = device.raw().allocate_descriptor_sets(&create_info);

            match result {
                Ok(set) => Ok(set[0]),
                Err(err_kind) => match err_kind {
                    vk::Result::ERROR_OUT_OF_POOL_MEMORY | vk::Result::ERROR_FRAGMENTED_POOL => {
                        self.pools.push(self.create_pool(device, layout)?);
                        self.allocate(device, layout)
                    }
                    _ => panic!("Descriptor Set allocation failed"),
                },
            }
        }
    }
    fn update_set(&self, device: &crate::Device, set: vk::DescriptorSet, layout: &DescriptorSetLayout, state: &DescriptorSetState) {
        hikari_dev::profile_function!();

        const MAX_WRITES: usize = MAX_BINDINGS_PER_SET * MAX_COUNTS_PER_BINDING;

        #[derive(Debug, Copy, Clone)]
        struct ImageWrite {
            binding: u32,
            ix: u32,
            image_info: vk::DescriptorImageInfo,
        }
        let mut image_writes = ArrayVecCopy::<ImageWrite, MAX_WRITES>::new();
        let mut storage_image_writes = ArrayVecCopy::<ImageWrite, MAX_WRITES>::new();

        #[derive(Debug, Copy, Clone)]
        struct BufferWrite {
            binding: u32,
            buffer_info: vk::DescriptorBufferInfo,
        }

        let mut ubo_writes = ArrayVecCopy::<BufferWrite, MAX_BINDINGS_PER_SET>::new();
        let mut storage_buffer_writes = ArrayVecCopy::<BufferWrite, MAX_BINDINGS_PER_SET>::new();

        crate::util::for_each_bit_in_range(
            layout.combined_image_sampler_mask(),
            0..MAX_BINDINGS_PER_SET,
            |binding| {
                let image_state = &state.bindings[binding as usize].image_state;

                crate::util::for_each_bit_in_range(
                    image_state.image_update_mask,
                    0..MAX_COUNTS_PER_BINDING,
                    |image_ix| {
                        let (image_view, sampler) = image_state.images[image_ix as usize];

                        image_writes.push(ImageWrite {
                            binding,
                            ix: image_ix,
                            image_info: *vk::DescriptorImageInfo::builder()
                                .image_view(image_view)
                                .sampler(sampler)
                                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL),
                        });
                    },
                );
            },
        );

        crate::util::for_each_bit_in_range(
            layout.storage_image_mask(),
            0..MAX_BINDINGS_PER_SET,
            |binding| {
                let image_state = &state.bindings[binding as usize].image_state;

                crate::util::for_each_bit_in_range(
                    image_state.image_update_mask,
                    0..MAX_COUNTS_PER_BINDING,
                    |image_ix| {
                        let (image_view, sampler) = image_state.images[image_ix as usize];

                        storage_image_writes.push(ImageWrite {
                            binding,
                            ix: image_ix,
                            image_info: *vk::DescriptorImageInfo::builder()
                                .image_view(image_view)
                                .sampler(sampler)
                                .image_layout(vk::ImageLayout::GENERAL),
                        });
                    },
                );
            },
        );

        crate::util::for_each_bit_in_range(
            layout.uniform_buffer_mask(),
            0..MAX_BINDINGS_PER_SET,
            |binding| {
                let buffer_state = &state.bindings[binding as usize].buffer_state;

                if buffer_state.buffer != vk::Buffer::null() {
                    ubo_writes.push(BufferWrite {
                        binding,
                        buffer_info: *vk::DescriptorBufferInfo::builder()
                            .buffer(buffer_state.buffer)
                            .offset(buffer_state.offset)
                            .range(buffer_state.range),
                    });
                }
            },
        );

        crate::util::for_each_bit_in_range(
            layout.storage_buffer_mask(),
            0..MAX_BINDINGS_PER_SET,
            |binding| {
                let buffer_state = &state.bindings[binding as usize].buffer_state;

                if buffer_state.buffer != vk::Buffer::null() {
                    storage_buffer_writes.push(BufferWrite {
                        binding,
                        buffer_info: *vk::DescriptorBufferInfo::builder()
                            .buffer(buffer_state.buffer)
                            .offset(buffer_state.offset)
                            .range(buffer_state.range),
                    });
                }
            },
        );

        let mut writes = ArrayVecCopy::<vk::WriteDescriptorSet, MAX_WRITES>::new();

        for write in &image_writes {
            let mut vk_write = *vk::WriteDescriptorSet::builder()
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .dst_set(set)
                .dst_binding(write.binding)
                .dst_array_element(write.ix);

            vk_write.p_image_info = &write.image_info;
            vk_write.descriptor_count = 1;

            writes.push(vk_write);
        }

        for write in &storage_image_writes {
            let mut vk_write = *vk::WriteDescriptorSet::builder()
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .dst_set(set)
                .dst_binding(write.binding)
                .dst_array_element(write.ix);

            vk_write.p_image_info = &write.image_info;
            vk_write.descriptor_count = 1;

            writes.push(vk_write);
        }

        for write in &ubo_writes {
            let mut vk_write = *vk::WriteDescriptorSet::builder()
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .dst_set(set)
                .dst_binding(write.binding)
                .dst_array_element(0);

            vk_write.p_buffer_info = &write.buffer_info;
            vk_write.descriptor_count = 1;

            writes.push(vk_write);
        }

        for write in &storage_buffer_writes {
            let mut vk_write = *vk::WriteDescriptorSet::builder()
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .dst_set(set)
                .dst_binding(write.binding)
                .dst_array_element(0);

            vk_write.p_buffer_info = &write.buffer_info;
            vk_write.descriptor_count = 1;

            writes.push(vk_write);
        }

        unsafe {
            hikari_dev::profile_scope!("Update descriptor set");
            //println!("Writes {:#?}", writes);
            device.raw().update_descriptor_sets(&writes, &[]);
        }
    }
    #[cold]
    fn create_and_update_set(
        &mut self,
        device: &crate::Device,
        hash: u64,
        layout: &DescriptorSetLayout,
        state: &DescriptorSetState,
    ) -> vk::DescriptorSet {
        hikari_dev::profile_function!();
        let new_set = self.resuable_sets.pop().unwrap_or_else(|| {
            self.allocate(device, layout)
                .expect("Failed to allocate descriptor set")
        });
        self.temp_map.insert(hash, new_set);

        self.update_set(device, new_set, layout, state);

        new_set
    }
    pub fn get(&mut self, device: &crate::Device, layout: &DescriptorSetLayout, state: &DescriptorSetState) -> vk::DescriptorSet {
        hikari_dev::profile_function!();

        //TODO: Investigate potential hash collisions
        let hash = state.hash(layout);

        match self.temp_map.get(&hash) {
            Some(&set) => set,
            None => self.create_and_update_set(device, hash, layout, state),
        }
    }
    pub fn into_pools(self) -> Vec<vk::DescriptorPool> {
        self.pools
    }
    fn new_frame(&mut self) {
        let reusable_sets = &mut self.resuable_sets;
        for removed_set in self.temp_map.new_frame() {
            reusable_sets.push(removed_set);
        }
    }
    pub fn delete(&mut self, device: &crate::Device) {
        for pool in self.pools.drain(..) {
            unsafe {
                device.raw().destroy_descriptor_pool(pool, None);
            }
        }
        log::debug!("Dropped DescriptorSetAllocator");
    }
}
pub struct DescriptorSetAllocator {
    device: Arc<crate::Device>,
    inner: RawDescriptorSetAllocator,
    set_layout: DescriptorSetLayout,

}
impl DescriptorSetAllocator {
    pub fn new(
        device: &Arc<crate::Device>,
        set_layout: DescriptorSetLayout,
    ) -> VkResult<Self> {
        Self::with_capacity(device, set_layout, MAX_SETS_PER_POOL)
    }
    pub fn with_capacity(device: &Arc<crate::Device>, set_layout: DescriptorSetLayout, max_sets_per_pool: usize) -> VkResult<Self> {
        let inner = RawDescriptorSetAllocator::with_capacity(device, &set_layout, max_sets_per_pool)?;

        Ok(Self {
            device: device.clone(),
            inner,
            set_layout,
        })
    }
    #[inline]
    pub fn allocate(&mut self) -> VkResult<vk::DescriptorSet> {
        self.inner.allocate(&self.device, &self.set_layout)
    } 
    #[inline]
    pub fn get(&mut self, state: &DescriptorSetState) -> vk::DescriptorSet {
        self.inner.get(&self.device, &self.set_layout, state)
    }
    #[inline]
    pub fn new_frame(&mut self) {
        self.inner.new_frame()
    }
}
impl Drop for DescriptorSetAllocator {
    fn drop(&mut self) {
        self.inner.delete(&self.device);
    }
}

unsafe impl Sync for DescriptorPool {}
unsafe impl Send for DescriptorPool {}

impl hikari_utils::hash::IsEnabled for VkDescriptorSetLayout {}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct VkDescriptorSetLayout(vk::DescriptorSetLayout);

pub struct DescriptorPool {
    device: Arc<crate::Device>,
    set_allocators: hikari_utils::hash::NoHashMap<
        VkDescriptorSetLayout,
        DescriptorSetAllocator,
    >,
}
impl DescriptorPool {
    pub fn new(device: &Arc<crate::Device>) -> Self {
        Self {
            device: device.clone(),
            set_allocators: HashMap::default(),
        }
    }
    pub fn get(&mut self, set_layout: &DescriptorSetLayout) -> &mut DescriptorSetAllocator {
        hikari_dev::profile_function!();

        match self
            .set_allocators
            .entry(VkDescriptorSetLayout(set_layout.raw()))
        {
            Entry::Occupied(allocator) => allocator.into_mut(),
            Entry::Vacant(vacant) => {
                vacant.insert(DescriptorSetAllocator::new(&self.device, *set_layout).unwrap())
            }
        }
    }
    pub fn new_frame(&mut self) {
        for allocator in self.set_allocators.values_mut() {
            allocator.new_frame();
        }
    }
}
