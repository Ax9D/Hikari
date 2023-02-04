use ash::{prelude::VkResult, vk};
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};

use std::{marker::PhantomData, sync::Arc, mem::ManuallyDrop};

pub trait Buffer {
    fn buffer(&self) -> vk::Buffer;
    fn byte_step(&self) -> vk::DeviceSize;
    fn len(&self) -> usize;
    fn size(&self) -> vk::DeviceSize;

    fn offset(&self, ix: usize) -> vk::DeviceSize {
        ix as u64 * self.byte_step()
    }
}
pub trait IndexType {
    fn index_type() -> vk::IndexType;
}

impl IndexType for GpuBuffer<u32> {
    #[inline(always)]
    fn index_type() -> vk::IndexType {
        vk::IndexType::UINT32
    }
}
impl IndexType for GpuBuffer<u16> {
    #[inline(always)]
    fn index_type() -> vk::IndexType {
        vk::IndexType::UINT16
    }
}

pub fn create_uniform_buffer<T: Copy>(
    device: &Arc<crate::Device>,
    len: usize,
) -> Result<CpuBuffer<T>, anyhow::Error> {
    CpuBuffer::new(
        device,
        len,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        gpu_allocator::MemoryLocation::CpuToGpu,
    )
}
pub fn create_vertex_buffer<T: Copy>(
    device: &Arc<crate::Device>,
    len: usize,
) -> Result<GpuBuffer<T>, anyhow::Error> {
    GpuBuffer::new(device, len, vk::BufferUsageFlags::VERTEX_BUFFER)
}
pub fn create_index_buffer(
    device: &Arc<crate::Device>,
    len: usize,
) -> Result<GpuBuffer<u32>, anyhow::Error> {
    GpuBuffer::new(device, len, vk::BufferUsageFlags::INDEX_BUFFER)
}

// pub struct ByteBuffer {
//     device: Arc<crate::Device>,
//     inner: vk::Buffer,
//     allocation: Allocation,
//     len: usize,
// }
// impl ByteBuffer {
//     pub fn new(device: &Arc<crate::Device>,
//         name: &str,
//         len: usize,
//         usage: vk::BufferUsageFlags,
//         location: gpu_allocator::MemoryLocation) -> Result<Self, anyhow::Error> {
//         let create_info = vk::BufferCreateInfo::builder()
//             .size((std::mem::size_of::<u8>() * len) as u64)
//             .usage(usage | vk::BufferUsageFlags::TRANSFER_DST)
//             .queue_family_indices(&[0])
//             .sharing_mode(vk::SharingMode::EXCLUSIVE);

//         let inner;
//         let requirements;

//         unsafe {
//             inner = device.raw().create_buffer(&create_info, None)?;
//             requirements = device.raw().get_buffer_memory_requirements(inner);
//         }

//         let allocation = device.allocate_memory(AllocationCreateDesc {
//             name,
//             requirements,
//             location,
//             linear: true,
//         })?;

//         Ok(Self {
//             device: device.clone(),
//             inner,
//             allocation,
//             len
//         })
//     }
//     pub fn raw(&self) -> vk::Buffer {
//         self.inner
//     }
//     pub fn data_ptr(&self) -> Option<NonNull<std::ffi::c_void>> {
//         self.allocation.mapped_ptr()
//     }
//     pub fn capacity(&self) -> usize {
//         self.len
//     }
// }
// impl Drop for ByteBuffer {
//     fn drop(&mut self) {
//         unsafe {
//             self.device.raw().destroy_buffer(self.inner, None);
//             self.device.free_memory(self.allocation.clone()).unwrap();
//         }
//     }
// }

// impl Buffer for ByteBuffer {
//     #[inline]
//     fn buffer(&self) -> vk::Buffer {
//         self.inner
//     }
//     #[inline]
//     fn byte_step(&self) -> vk::DeviceSize {
//         std::mem::size_of::<u8>() as u64
//     }
//     #[inline]
//     fn size(&self) -> vk::DeviceSize {
//         self.offset(self.capacity())
//     }

//     fn len(&self) -> usize {
//        self.capacity()
//     }
// }

pub struct CpuBuffer<T> {
    device: Arc<crate::Device>,
    inner: vk::Buffer,
    allocation: ManuallyDrop<Allocation>,
    len: usize,
    _phantom: PhantomData<T>,
}

impl<T: Copy> CpuBuffer<T> {
    pub fn new(
        device: &Arc<crate::Device>,
        len: usize,
        usage: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
    ) -> Result<Self, anyhow::Error> {
        if location == gpu_allocator::MemoryLocation::GpuOnly
            || location == gpu_allocator::MemoryLocation::Unknown
        {
            panic!(
                "MemoryLocation must be CpuToGpu or GpuToCpu: {:?} is unsupported for CpuBuffer",
                location
            );
        }

        let create_info = vk::BufferCreateInfo::builder()
            .size((std::mem::size_of::<T>() * len) as u64)
            .usage(usage)
            .queue_family_indices(&[0])
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let inner;
        let requirements;

        unsafe {
            inner = device.raw().create_buffer(&create_info, None)?;
            requirements = device.raw().get_buffer_memory_requirements(inner);
        }

        let allocation = device.allocate_memory(AllocationCreateDesc {
            name: "cpu_buffer",
            requirements,
            location,
            linear: true,
            allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
        })?;

        unsafe {
            device
                .raw()
                .bind_buffer_memory(inner, allocation.memory(), allocation.offset())?;
        }

        Ok(Self {
            device: device.clone(),
            inner,
            allocation: ManuallyDrop::new(allocation),
            _phantom: PhantomData::default(),
            len,
        })
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.len
    }
    pub fn mapped_slice(&self) -> &[T] {
        let ptr = self.allocation.mapped_ptr().unwrap().as_ptr(); //Host coherent so no invalidate

        unsafe { std::slice::from_raw_parts(ptr as *const T, self.capacity()) }
    }
    pub fn mapped_slice_mut(&mut self) -> &mut [T] {
        let ptr = self.allocation.mapped_ptr().unwrap().as_ptr(); //Host coherent so no invalidate

        unsafe { std::slice::from_raw_parts_mut(ptr as *mut T, self.capacity()) }
    }

    // TODO: Check Alignment
    // pub fn cast<U: Pod>(self) -> CpuBuffer<U> {
    //     let len_u = std::mem::size_of::<T>() * self.len() / std::mem::size_of::<U>();

    //     CpuBuffer {
    //         device: self.device,
    //         inner: self.inner,
    //         allocation: self.allocation,
    //         _phantom: PhantomData::default(),
    //         len: len_u,
    //     }
    // }
}

impl<T> Drop for CpuBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_buffer(self.inner, None);
            let allocation = ManuallyDrop::take(&mut self.allocation);
            self.device.free_memory(allocation).unwrap();
        }
    }
}

impl<T: Copy> Buffer for CpuBuffer<T> {
    #[inline]
    fn buffer(&self) -> vk::Buffer {
        self.inner
    }
    #[inline]
    fn byte_step(&self) -> vk::DeviceSize {
        std::mem::size_of::<T>() as u64
    }
    #[inline]
    fn size(&self) -> vk::DeviceSize {
        self.offset(self.capacity())
    }

    fn len(&self) -> usize {
        self.capacity()
    }
}

pub struct GpuBuffer<T> {
    device: Arc<crate::Device>,
    inner: vk::Buffer,
    allocation: ManuallyDrop<Allocation>,
    len: usize,
    _phantom: PhantomData<T>,
}

impl<T: Copy> GpuBuffer<T> {
    pub fn new(
        device: &Arc<crate::Device>,
        len: usize,
        usage: vk::BufferUsageFlags,
    ) -> Result<Self, anyhow::Error> {
        let create_info = vk::BufferCreateInfo::builder()
            .size((std::mem::size_of::<T>() * len) as u64)
            .usage(usage | vk::BufferUsageFlags::TRANSFER_DST)
            .queue_family_indices(&[0])
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let inner;
        let requirements;

        unsafe {
            inner = device.raw().create_buffer(&create_info, None)?;
            requirements = device.raw().get_buffer_memory_requirements(inner);
        }

        let allocation = device.allocate_memory(AllocationCreateDesc {
            name: "gpu_buffer",
            requirements,
            location: gpu_allocator::MemoryLocation::GpuOnly,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged
        })?;

        unsafe {
            device
                .raw()
                .bind_buffer_memory(inner, allocation.memory(), allocation.offset())?;
        }

        Ok(Self {
            device: device.clone(),
            inner,
            allocation: ManuallyDrop::new(allocation),
            len,
            _phantom: PhantomData,
        })
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.len
    }

    ///Copies the data from the Host to the GPU, no synchronization is performed on the GPU side, the caller must ensure the buffer is not being used on the GPU
    ///Offset is the count of T from index 0, not bytes
    pub fn upload(&mut self, data: &[T], offset_ix: usize) -> VkResult<()> {
        let mut upload_buffer = CpuBuffer::<T>::new(
            &self.device,
            self.len,
            vk::BufferUsageFlags::TRANSFER_SRC,
            gpu_allocator::MemoryLocation::CpuToGpu,
        )
        .unwrap();

        upload_buffer.mapped_slice_mut()[0..data.len()].copy_from_slice(data);

        unsafe {
            let device = self.device.raw().clone();
            self.device.submit_commands_immediate(|cmd| {
                let src_buffer = upload_buffer.buffer();
                let dst_buffer = self.buffer();

                let src_offset = upload_buffer.offset(0);
                let dst_offset = self.offset(offset_ix);

                let size = (data.len() * std::mem::size_of::<T>()) as u64;

                let regions = [vk::BufferCopy {
                    src_offset,
                    dst_offset,
                    size,
                }];

                device.cmd_copy_buffer(cmd, src_buffer, dst_buffer, &regions);
                Ok(())
            })
        }
    }

    // pub fn cast<U: Pod>(self) -> GpuBuffer<U> {
    //     GpuBuffer {
    //         device: self.device,
    //         inner: self.inner,
    //         allocation: self.allocation,
    //         staging_buffer: self.staging_buffer.cast(),
    //     }
    // }
}

impl<T> Drop for GpuBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_buffer(self.inner, None);
            let allocation = ManuallyDrop::take(&mut self.allocation);

            self.device.free_memory(allocation).unwrap();
        }
    }
}

impl<T: Copy> Buffer for GpuBuffer<T> {
    #[inline]
    fn buffer(&self) -> vk::Buffer {
        self.inner
    }
    #[inline]
    fn byte_step(&self) -> vk::DeviceSize {
        std::mem::size_of::<T>() as u64
    }
    #[inline]
    fn size(&self) -> vk::DeviceSize {
        self.offset(self.capacity())
    }

    fn len(&self) -> usize {
        self.capacity()
    }
}

pub struct UniformBuffer<T> {
    inner: [CpuBuffer<T>; 2],
    frame: usize,
}

impl<T: Copy> UniformBuffer<T> {
    pub fn new(device: &Arc<crate::Device>, len: usize) -> anyhow::Result<Self> {
        let inner = [
            CpuBuffer::new(
                device,
                len,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                gpu_allocator::MemoryLocation::CpuToGpu,
            )?,
            CpuBuffer::new(
                device,
                len,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                gpu_allocator::MemoryLocation::CpuToGpu,
            )?,
        ];
        Ok(Self { inner, frame: 0 })
    }
    #[inline]
    pub fn mapped_slice(&self) -> &[T] {
        self.inner[self.frame].mapped_slice()
    }
    #[inline]
    pub fn mapped_slice_mut(&mut self) -> &mut [T] {
        self.inner[self.frame].mapped_slice_mut()
    }
    pub fn new_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1) % 2;
    }
}

impl<T: Copy> Buffer for UniformBuffer<T> {
    fn buffer(&self) -> vk::Buffer {
        self.inner[self.frame].buffer()
    }

    fn byte_step(&self) -> vk::DeviceSize {
        self.inner[0].byte_step()
    }

    fn size(&self) -> vk::DeviceSize {
        self.inner[0].size()
    }

    fn len(&self) -> usize {
        self.inner[0].capacity()
    }
}
