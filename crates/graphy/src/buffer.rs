use ash::{prelude::VkResult, vk};
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc};

use crate::{shader::ShaderDataType, ResourceAllocationError};

use std::{
    io::IntoInnerError,
    marker::PhantomData,
    ops::{Index, IndexMut},
    sync::Arc,
};

pub trait SupportedPrimitive {}

impl SupportedPrimitive for i32 {}
impl SupportedPrimitive for f32 {}
impl SupportedPrimitive for bool {}

pub struct ImmutableVertexBuffer {
    id: u32,
    layout: Vec<ShaderDataType>,
}

pub trait Buffer {}

impl ImmutableVertexBuffer {}

pub struct UniformBuffer<T> {
    id: u32,
    size: usize,
    _phantom: PhantomData<T>,
}
impl<T> UniformBuffer<T> {}
impl<T> Buffer for UniformBuffer<T> {}

pub struct IndexBuffer {
    id: u32,
}

impl IndexBuffer {}

//---------------------------------------

// pub struct MappedSlice<'a, T: Copy> {
//     slice: &'a [T],
//     count: usize,
//     offset: usize,
//     parent: &'a UploadBuffer<T>
// }
// // impl<'a, T> Index<usize> for MappedSlice<'a, T> {
// //     type Output = T;

// //     fn index(&self, index: usize) -> &Self::Output {
// //         self.slice.index(index)
// //     }
// // }

// impl<'a, T: Copy> AsRef<[T]> for MappedSlice<'a, T> {
//     fn as_ref(&self) -> &[T] {
//         self.slice
//     }
// }

// impl<'a, T: Copy> Drop for MappedSlice<'a, T> {
//     fn drop(&mut self) {
//        unsafe { UploadBuffer::unmap_raw(self.parent, self.count, self.offset).unwrap(); }
//     }
// }

// pub struct MappedSliceMut<'a, T: Copy> {
//     slice: &'a mut [T],
//     count: usize,
//     offset: usize,
//     parent: &'a UploadBuffer<T>
// }

// impl<'a, T: Copy> AsRef<[T]> for MappedSliceMut<'a, T> {
//     fn as_ref(&self) -> &[T] {
//         self.slice
//     }
// }

// impl<'a, T: Copy> AsMut<[T]> for MappedSliceMut<'a, T> {
//     fn as_mut(&mut self) -> &mut [T] {
//         self.slice
//     }
// }

// // impl<'a, T> Index<usize> for MappedSliceMut<'a, T> {
// //     type Output = T;

// //     fn index(&self, index: usize) -> &Self::Output {
// //         self.slice.index(index)
// //     }
// // }
// // impl<'a, T> IndexMut<usize> for MappedSliceMut<'a, T> {
// //     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
// //         self.slice.index_mut(index)
// //     }
// // }

// impl<'a, T: Copy> Drop for MappedSliceMut<'a, T> {
//     fn drop(&mut self) {
//        unsafe { UploadBuffer::unmap_raw(self.parent, self.count, self.offset).unwrap(); }
//     }
// }

pub(crate) struct CpuBuffer<T> {
    device: Arc<crate::Device>,
    inner: vk::Buffer,
    allocation: Allocation,
    _phantom: PhantomData<T>,
    len: usize,
}

impl<T: Copy> CpuBuffer<T> {
    pub fn new(
        device: &Arc<crate::Device>,
        len: usize,
        usage: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
        sharing_mode: vk::SharingMode,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let create_info = vk::BufferCreateInfo::builder()
            .size((std::mem::size_of::<T>() * len) as u64)
            .usage(usage)
            .sharing_mode(sharing_mode);

        let inner;
        let requirements;

        unsafe {
            inner = device.raw().create_buffer(&create_info, None)?;
            requirements = device.raw().get_buffer_memory_requirements(inner);
        }

        let allocation = device.allocate_memory(AllocationCreateDesc {
            name: "buffer",
            requirements,
            location,
            linear: true,
        })?;

        unsafe {
            device
                .raw()
                .bind_buffer_memory(inner, allocation.memory(), allocation.offset())?;
        }

        Ok(Self {
            device: device.clone(),
            inner,
            allocation,
            _phantom: PhantomData::default(),
            len,
        })
    }
    pub fn raw(&self) -> vk::Buffer {
        self.inner
    }
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn mapped_slice<'a>(&'a self) -> &'a [T] {
        let ptr = self.allocation.mapped_ptr().unwrap().as_ptr(); //Host coherent so no invalidate

        unsafe { std::slice::from_raw_parts(ptr as *const T, self.len()) }
    }
    pub fn mapped_slice_mut<'a>(&'a mut self) -> &'a mut [T] {
        let ptr = self.allocation.mapped_ptr().unwrap().as_ptr(); //Host coherent so no invalidate

        unsafe { std::slice::from_raw_parts_mut(ptr as *mut T, self.len()) }
    }
}

impl<T> Drop for CpuBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_buffer(self.inner, None);
            self.device.free_memory(self.allocation.clone()).unwrap();
        }
        log::debug!("Dropped buffer");
    }
}
