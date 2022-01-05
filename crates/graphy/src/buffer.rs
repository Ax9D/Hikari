use ash::{prelude::VkResult, vk};
use bytemuck::Pod;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc};

use std::{marker::PhantomData, sync::Arc};

pub trait Buffer {
    fn buffer(&self) -> vk::Buffer;
    fn base_offset(&self) -> vk::DeviceSize;
    fn byte_step(&self) -> vk::DeviceSize;
    fn size(&self) -> vk::DeviceSize;

    fn offset(&self, ix: usize) -> vk::DeviceSize {
        self.base_offset() + ix as u64 * self.byte_step()
    }
}

pub fn create_uniform_buffer<T: Pod>(
    device: &Arc<crate::Device>,
    len: usize,
) -> Result<CpuBuffer<T>, Box<dyn std::error::Error>> {
    CpuBuffer::new(
        device,
        len,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        gpu_allocator::MemoryLocation::CpuToGpu,
    )
}
pub fn create_vertex_buffer<T: Pod>(
    device: &Arc<crate::Device>,
    len: usize,
) -> Result<GpuBuffer<T>, Box<dyn std::error::Error>> {
    GpuBuffer::new(device, len, vk::BufferUsageFlags::VERTEX_BUFFER)
}
pub fn create_index_buffer<T: Pod>(
    device: &Arc<crate::Device>,
    len: usize,
) -> Result<GpuBuffer<T>, Box<dyn std::error::Error>> {
    GpuBuffer::new(device, len, vk::BufferUsageFlags::INDEX_BUFFER)
}

pub struct CpuBuffer<T> {
    device: Arc<crate::Device>,
    inner: vk::Buffer,
    allocation: Allocation,
    len: usize,
    _phantom: PhantomData<T>,
}

impl<T: Pod> CpuBuffer<T> {
    pub fn new(
        device: &Arc<crate::Device>,
        len: usize,
        usage: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn mapped_slice<'a>(&'a self) -> &'a [T] {
        let ptr = self.allocation.mapped_ptr().unwrap().as_ptr(); //Host coherent so no invalidate

        unsafe { std::slice::from_raw_parts(ptr as *const T, self.len()) }
    }
    pub fn mapped_slice_mut<'a>(&'a mut self) -> &'a mut [T] {
        let ptr = self.allocation.mapped_ptr().unwrap().as_ptr(); //Host coherent so no invalidate

        unsafe { std::slice::from_raw_parts_mut(ptr as *mut T, self.len()) }
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
            self.device.free_memory(self.allocation.clone()).unwrap();
        }
        log::debug!("Dropped CPUBuffer");
    }
}

impl<T: Pod> Buffer for CpuBuffer<T> {
    #[inline]
    fn buffer(&self) -> vk::Buffer {
        self.inner
    }
    #[inline]
    fn base_offset(&self) -> vk::DeviceSize {
        self.allocation.offset()
    }
    #[inline]
    fn byte_step(&self) -> vk::DeviceSize {
        std::mem::size_of::<T>() as u64
    }
    #[inline]
    fn size(&self) -> vk::DeviceSize {
        self.offset(self.len())
    }
}

pub struct GpuBuffer<T> {
    device: Arc<crate::Device>,
    inner: vk::Buffer,
    allocation: Allocation,
    upload_buffer: CpuBuffer<T>,
}

impl<T: Pod> GpuBuffer<T> {
    pub fn new(
        device: &Arc<crate::Device>,
        len: usize,
        usage: vk::BufferUsageFlags,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let upload_buffer = CpuBuffer::<T>::new(
            device,
            len,
            vk::BufferUsageFlags::TRANSFER_SRC,
            gpu_allocator::MemoryLocation::CpuToGpu,
        )?;

        let create_info = vk::BufferCreateInfo::builder()
            .size((std::mem::size_of::<T>() * len) as u64)
            .usage(usage | vk::BufferUsageFlags::TRANSFER_DST)
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
            upload_buffer,
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.upload_buffer.len()
    }

    ///Copies the data from the Host to the GPU, no synchronization is performed on the GPU side, the caller must ensure the buffer is not being used on the GPU
    ///Offset is the count of T from index 0, not bytes
    pub fn upload(&mut self, data: &[T], offset_ix: usize) -> VkResult<()> {
        self.upload_buffer.mapped_slice_mut()[0..data.len()].copy_from_slice(data);

        unsafe {
            let device = self.device.raw().clone();
            self.device.submit_commands_immediate(|cmd| {
                let src_buffer = self.upload_buffer.buffer();
                let dst_buffer = self.buffer();

                let src_offset = self.upload_buffer.offset(0);
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
            self.device.free_memory(self.allocation.clone()).unwrap();
        }
        log::debug!("Dropped GPUBuffer");
    }
}

impl<T: Pod> Buffer for GpuBuffer<T> {
    #[inline]
    fn buffer(&self) -> vk::Buffer {
        self.inner
    }
    #[inline]
    fn base_offset(&self) -> vk::DeviceSize {
        self.allocation.offset()
    }
    #[inline]
    fn byte_step(&self) -> vk::DeviceSize {
        std::mem::size_of::<T>() as u64
    }
    #[inline]
    fn size(&self) -> vk::DeviceSize {
        self.offset(self.len())
    }
}
