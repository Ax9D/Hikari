use ash::{prelude::VkResult, vk};
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc},
    AllocationError,
};

use std::{marker::PhantomData, mem::ManuallyDrop, sync::Arc, ops::Range};

use crate::bindless::BindlessHandle;

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
) -> Result<RingBuffer<T>, anyhow::Error> {
    RingBuffer::new(
        device,
        len,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        gpu_allocator::MemoryLocation::CpuToGpu,
    )
}
/// Creates a storage buffer on the CPU side using a ring buffer
pub fn create_storage_buffer<T: Copy>(device: &Arc<crate::Device>, len: usize) -> Result<RingBuffer<T>, anyhow::Error> {
    RingBuffer::new(
        device,
        len,
        vk::BufferUsageFlags::STORAGE_BUFFER,
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

pub(crate) struct RawBuffer<T> {
    inner: vk::Buffer,
    allocation: ManuallyDrop<Allocation>,
    len: usize,
    _phantom: PhantomData<T>,
}
impl<T: Copy> RawBuffer<T> {
    pub fn new(
        device: &crate::Device,
        name: &str,
        len: usize,
        usage: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
    ) -> Result<Self, anyhow::Error> {
        let create_info = vk::BufferCreateInfo::default()
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
            name,
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
            inner,
            allocation: ManuallyDrop::new(allocation),
            _phantom: PhantomData::default(),
            len,
        })
    } 
    pub fn buffer(&self) -> vk::Buffer {
        self.inner
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
}
impl<T> RawBuffer<T> {
    pub fn delete(&mut self, device: &crate::Device) {
        use crate::delete::DeleteRequest;
        let deleter = device.deleter();
        let allocation = unsafe { ManuallyDrop::take(&mut self.allocation) };
        deleter.request_delete(DeleteRequest::Buffer(self.inner, allocation));
    }
}

pub fn delete_buffer(
    device: &crate::Device,
    buffer: vk::Buffer,
    allocation: Allocation,
) -> Result<(), AllocationError> {
    unsafe {
        device.raw().destroy_buffer(buffer, None);
    };
    device.free_memory(allocation)
}
pub struct CpuBuffer<T> {
    device: Arc<crate::Device>,
    raw: RawBuffer<T>
}

impl<T: Copy> CpuBuffer<T> {
    pub fn new(
        device: &Arc<crate::Device>,
        len: usize,
        usage: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
    ) -> Result<Self, anyhow::Error> {
        let name = "cpu_buffer";
        if location == gpu_allocator::MemoryLocation::GpuOnly
            || location == gpu_allocator::MemoryLocation::Unknown
        {
            panic!(
                "MemoryLocation must be CpuToGpu or GpuToCpu: {:?} is unsupported for CpuBuffer",
                location
            );
        }
        let raw = RawBuffer::new(device, name, len, usage, location)?;

        Ok(Self {
            device: device.clone(),
            raw,
        })
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }
    #[inline]
    pub fn mapped_slice(&self) -> &[T] {
        self.raw.mapped_slice()
    }
    #[inline]
    pub fn mapped_slice_mut(&mut self) -> &mut [T] {
        self.raw.mapped_slice_mut()
    }
}

impl<T> Drop for CpuBuffer<T> {
    fn drop(&mut self) {
        self.raw.delete(&self.device);
    }
}

impl<T: Copy> Buffer for CpuBuffer<T> {
    #[inline]
    fn buffer(&self) -> vk::Buffer {
        self.raw.buffer()
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
    raw: RawBuffer<T>
}

impl<T: Copy> GpuBuffer<T> {
    pub fn new(
        device: &Arc<crate::Device>,
        len: usize,
        usage: vk::BufferUsageFlags,
    ) -> Result<Self, anyhow::Error> {
        let name = "gpu_buffer";
        let usage = usage | vk::BufferUsageFlags::TRANSFER_DST;
        let location = gpu_allocator::MemoryLocation::GpuOnly;

        let raw = RawBuffer::new(device, name, len, usage, location)?;

        Ok(Self {
            device: device.clone(),
            raw,
        })
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    ///Copies the data from the Host to the GPU, no synchronization is performed on the GPU side, the caller must ensure the buffer is not being used on the GPU
    ///Offset is the count of T from index 0, not bytes
    pub fn upload(&mut self, data: &[T], offset_ix: usize) -> VkResult<()> {
        let mut upload_buffer = CpuBuffer::<T>::new(
            &self.device,
            self.capacity(),
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

    ///Copies the data from the GPU to the Host, no synchronization is performed on the GPU side, the caller must ensure the buffer is not being used on the GPU
    pub fn download(&self, range: Range<usize>) -> VkResult<CpuBuffer<T>> {
        let download_buffer = CpuBuffer::<T>::new(
            &self.device,
            range.len(),
            vk::BufferUsageFlags::TRANSFER_DST,
            gpu_allocator::MemoryLocation::GpuToCpu,
        )
        .unwrap();

        let offset = range.start;
        unsafe {
            let device = self.device.raw().clone();
            self.device.submit_commands_immediate(|cmd| {
                let src_buffer = self.buffer();
                let dst_buffer = download_buffer.buffer();

                let src_offset = self.offset(offset);
                let dst_offset = download_buffer.offset(0);

                let size = (range.len() * std::mem::size_of::<T>()) as u64;

                let regions = [vk::BufferCopy {
                    src_offset,
                    dst_offset,
                    size,
                }];

                device.cmd_copy_buffer(cmd, src_buffer, dst_buffer, &regions);
                Ok(())
            })?;
        }

        Ok(download_buffer)
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
        self.raw.delete(&self.device);
    }
}

impl<T: Copy> Buffer for GpuBuffer<T> {
    #[inline]
    fn buffer(&self) -> vk::Buffer {
        self.raw.buffer()
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
pub struct RingBuffer<T> {
    inner: [CpuBuffer<T>; 2],
    bindless_handles: Option<[BindlessHandle<vk::Buffer>; 2]>,
    frame: usize,
}
impl<T: Copy> RingBuffer<T> {
    pub fn new(device: &Arc<crate::Device>, len: usize, usage: vk::BufferUsageFlags, location: gpu_allocator::MemoryLocation) -> anyhow::Result<Self> {
        let inner = [CpuBuffer::new(device, len, usage, location)?, CpuBuffer::new(device, len, usage, location)?];

        let bindless_handles;

        if usage.intersects(vk::BufferUsageFlags::STORAGE_BUFFER) {
            let bindless = device.bindless_resources();
            let handle1 = bindless.allocate_buffer(device, inner[0].buffer(), inner[0].size(), usage);
            let handle2 = bindless.allocate_buffer(device, inner[1].buffer(), inner[1].size(), usage);

            bindless_handles = Some([handle1, handle2]);
        } else {
            bindless_handles = None;
        }

        Ok(Self { inner, bindless_handles, frame: 0 })
    }
    #[inline]
    pub fn mapped_slice(&self) -> &[T] {
        self.inner[self.frame].mapped_slice()
    }
    #[inline]
    pub fn mapped_slice_mut(&mut self) -> &mut [T] {
        self.inner[self.frame].mapped_slice_mut()
    }
    #[inline]
    pub fn bindless_handle(&mut self) -> Option<BindlessHandle<vk::Buffer>> {
        self.bindless_handles.map(|handles| handles[self.frame])
    }
    pub fn new_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1) % 2;
    }
}
impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        let device = &self.inner[0].device;
        let deleter = device.deleter();

        use crate::delete::DeleteRequest;
        if let Some(handles) = self.bindless_handles {
            deleter.request_delete(DeleteRequest::BindlessBuffer(handles[0]));
            deleter.request_delete(DeleteRequest::BindlessBuffer(handles[1]));
        }
    }
}
impl<T: Copy> Buffer for RingBuffer<T> {
    fn buffer(&self) -> vk::Buffer {
        self.inner[self.frame].buffer()
    }

    fn byte_step(&self) -> vk::DeviceSize {
        self.inner[self.frame].byte_step()
    }

    fn size(&self) -> vk::DeviceSize {
        self.inner[self.frame].size()
    }

    fn len(&self) -> usize {
        self.inner[self.frame].capacity()
    }
}