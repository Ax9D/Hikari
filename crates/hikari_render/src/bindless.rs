use std::{sync::{atomic::{AtomicUsize, Ordering}}, marker::PhantomData, num::NonZeroUsize};

use ash::{vk::{self}};
use parking_lot::Mutex;

use crate::{descriptor::{DescriptorSetLayout, RawDescriptorSetAllocator, MAX_BINDLESS_COUNT}, ImageConfig, RawSampledImage};

struct IndexAllocator {
    new_index: AtomicUsize,
    freed_indices: flume::Receiver<NonZeroUsize>,
    freed_indices_sender: flume::Sender<NonZeroUsize>,
}

impl IndexAllocator {
    pub fn new() -> Self {
        let (freed_indices_sender, freed_indices) = flume::unbounded();
        Self {
            new_index: AtomicUsize::new(1),
            freed_indices,
            freed_indices_sender
        }
    }
    pub fn allocate(&self) -> NonZeroUsize {
        self.freed_indices
            .try_iter()
            .next()
            .unwrap_or_else(|| {
                let val = self.new_index.fetch_add(1, Ordering::SeqCst);

                // Safe because new_index was initialized with 1
                unsafe {
                    NonZeroUsize::new_unchecked(val)
                }
            })
    }
    pub fn deallocate(&self, index: NonZeroUsize) {
        self.freed_indices_sender
            .send(index)
            .expect("Failed to send")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BindlessHandle<T>(NonZeroUsize, PhantomData<T>);

impl<T> BindlessHandle<T> {
    fn from_raw(index: NonZeroUsize) -> Self {
        Self(index, PhantomData)
    }
    pub fn index(&self) -> usize {
        self.0.into()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BindlessResource {
    CombinedImageSampler = 0,
    StorageImage,
    UniformBuffer,
    StorageBuffer,
}
impl BindlessResource {
    pub fn vk_descriptor_type(self) -> vk::DescriptorType {
        match self {
            BindlessResource::CombinedImageSampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            BindlessResource::StorageImage => vk::DescriptorType::STORAGE_IMAGE,
            BindlessResource::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
            BindlessResource::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
        }
    }
}
pub struct BindlessResources {
    set: Mutex<vk::DescriptorSet>,
    image_indices: IndexAllocator,
    buffer_indices: IndexAllocator,
    //buffer_indices: IndexAllocator,
    
    allocator: RawDescriptorSetAllocator,
    layout: DescriptorSetLayout,
    debug_image: RawSampledImage,
}

impl BindlessResources {
    pub fn new(device: &crate::Device) -> anyhow::Result<Self> {
        let mut layout = DescriptorSetLayout::builder();
        
        let binding_flags = 
        vk::DescriptorBindingFlags::PARTIALLY_BOUND | 
        vk::DescriptorBindingFlags::UPDATE_AFTER_BIND | 
        vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING;
        
        layout
        .create_flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
    
    let bindings = [BindlessResource::CombinedImageSampler, BindlessResource::StorageImage, BindlessResource::StorageBuffer];
    
    for binding in bindings {
        layout.with_binding(binding as u32, binding.vk_descriptor_type(), MAX_BINDLESS_COUNT as u32, vk::ShaderStageFlags::ALL, binding_flags);
    }
    let layout = layout.build(device)?;
    
        let mut allocator = RawDescriptorSetAllocator::with_capacity(device, &layout, 1)?;

        let set = Mutex::new(allocator.allocate(device, &layout)?);
        
        println!("Bindless set: {:?}", set.lock());

        let config = ImageConfig::default();

        let debug_image = RawSampledImage::with_data(device, &[255u8, 0, 255, 255], 1, 1, 1, config)?;
        let image_indices = IndexAllocator::new();
        let buffer_indices = IndexAllocator::new();
        let bindless = Self {
            allocator,
            set, 
            layout,
            image_indices,
            buffer_indices,
            debug_image
        };

        #[cfg(debug_assertions)]
        for i in 0..MAX_BINDLESS_COUNT {
            bindless.set_debug_image(device, BindlessResource::CombinedImageSampler as u32, i);
        }

        Ok(bindless)
    }
    pub fn set(&self) -> &Mutex<vk::DescriptorSet> {
        &self.set
    }
    pub fn set_layout(&self) -> &DescriptorSetLayout {
        &self.layout
    }
    fn set_debug_image(&self, device: &crate::Device, binding: u32, index: usize) {
        self.update_image_view(device, binding, index, 
        self.debug_image.shader_resource_view(0).unwrap(),
         self.debug_image.sampler(), 
         vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, 
         vk::DescriptorType::COMBINED_IMAGE_SAMPLER);
    }
    pub fn allocate_image(&self, device: &crate::Device, image_view: vk::ImageView, sampler: vk::Sampler, usage: vk::ImageUsageFlags) -> BindlessHandle<vk::ImageView> {        
        let indices = &self.image_indices;
        let index = indices.allocate();
        let handle = BindlessHandle::from_raw(index);
        
        let is_sampled = usage.contains(vk::ImageUsageFlags::SAMPLED);
        let is_storage = usage.contains(vk::ImageUsageFlags::STORAGE);

        if is_sampled {
            self.set_combined_image_sampler(device, &handle, image_view, sampler);
        }

        if is_storage {
            self.set_storage_image(device, &handle, image_view);
        }

        handle
    }
    pub fn deallocate_image(&self, device: &crate::Device, handle: BindlessHandle<vk::ImageView>) {
        self.image_indices.deallocate(handle.0);

        #[cfg(debug_assertions)]
        self.set_debug_image(device, BindlessResource::CombinedImageSampler as u32, handle.index());
    } 
    pub fn allocate_buffer(&self, device: &crate::Device, buffer: vk::Buffer, buffer_size_bytes: u64, usage: vk::BufferUsageFlags) -> BindlessHandle<vk::Buffer> {
        let indices = &self.image_indices;
        let index = indices.allocate();

        //let is_uniform_buffer = usage.contains(vk::BufferUsageFlags::UNIFORM_BUFFER);
        let is_storage_buffer = usage.contains(vk::BufferUsageFlags::STORAGE_BUFFER);

        assert!(is_storage_buffer);

        let handle = BindlessHandle::from_raw(index);

        
        // if is_uniform_buffer {
        //     self.set_uniform_buffer(device, &handle, buffer, buffer_size_bytes);
        // }

        if is_storage_buffer {
            self.set_storage_buffer(device, &handle, buffer, buffer_size_bytes);
        }

        handle
    }
    pub fn deallocate_buffer(&self, device: &crate::Device, handle: BindlessHandle<vk::Buffer>) {
        self.buffer_indices.deallocate(handle.0)
    }
    fn set_combined_image_sampler(&self, device: &crate::Device, handle: &BindlessHandle<vk::ImageView>, image_view: vk::ImageView, sampler: vk::Sampler) {
        let resource = BindlessResource::CombinedImageSampler;
        let binding = resource as u32;
        let desc_type = resource.vk_descriptor_type();

        self.update_image_view(device, binding, handle.index(), image_view, sampler, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, desc_type);
    } 
    fn set_storage_image(&self, device: &crate::Device, handle: &BindlessHandle<vk::ImageView>, image_view: vk::ImageView) {
        let resource = BindlessResource::StorageImage;
        let binding = resource as u32;
        let desc_type = resource.vk_descriptor_type();
        
        self.update_image_view(device, binding, handle.index(), image_view, vk::Sampler::null(), vk::ImageLayout::GENERAL, desc_type);
    }
    fn set_uniform_buffer(&self, device: &crate::Device, handle: &BindlessHandle<vk::Buffer>, buffer: vk::Buffer, buffer_size_bytes: u64) {
        let resource = BindlessResource::UniformBuffer;
        let binding = resource as u32;
        let desc_type = resource.vk_descriptor_type();
        
        self.update_buffer(device, binding, handle.index(), buffer, buffer_size_bytes, desc_type);
    }
    fn set_storage_buffer(&self, device: &crate::Device, handle: &BindlessHandle<vk::Buffer>, buffer: vk::Buffer, buffer_size_bytes: u64) {
        let resource = BindlessResource::StorageBuffer;
        let binding = resource as u32;
        let desc_type = resource.vk_descriptor_type();
        
        self.update_buffer(device, binding, handle.index(), buffer, buffer_size_bytes, desc_type);
    }
    fn update_buffer(&self, device: &crate::Device, binding: u32, index: usize, buffer: vk::Buffer, buffer_size_bytes: u64, desc_type: vk::DescriptorType) {
        let buffer_info = [vk::DescriptorBufferInfo::default()
        .buffer(buffer)
        .offset(0)
        .range(buffer_size_bytes)
        ];

        let set = self.set.lock();

        let descriptor_writes = [vk::WriteDescriptorSet::default()
        .dst_set(*set)
        .buffer_info(&buffer_info)
        .dst_binding(binding)
        .descriptor_type(desc_type)
        .dst_array_element(index as u32)];

        unsafe {
            device.raw().update_descriptor_sets(&descriptor_writes, &[]);
        }
    }
    fn update_image_view(&self, device: &crate::Device, binding: u32, index: usize, image_view: vk::ImageView, sampler: vk::Sampler, layout: vk::ImageLayout, desc_type: vk::DescriptorType) {
        let image_info = [vk::DescriptorImageInfo::default()
        .image_view(image_view)
        .sampler(sampler)
        .image_layout(layout)
        ];

        let set = self.set.lock();

        let descriptor_writes = [vk::WriteDescriptorSet::default()
        .dst_set(*set)
        .image_info(&image_info)
        .dst_binding(binding)
        .descriptor_type(desc_type)
        .dst_array_element(index as u32)];

        unsafe {
            device.raw().update_descriptor_sets(&descriptor_writes, &[]);
        }
    }
    pub fn delete(&mut self, device: &crate::Device) {
        self.allocator.delete(device);
        self.debug_image.delete_now(device);
    }
}
