use ash::vk;
use gpu_allocator::vulkan::AllocationScheme;
use gpu_allocator::{vulkan::Allocation, vulkan::AllocationCreateDesc, AllocationError};

mod sampled_image;
mod sampler;
mod view;
mod config;
mod raw;

pub use raw::*;
pub use sampled_image::*;
pub use sampler::*;
pub use view::*;
pub use config::*;

pub fn create_image(
    device: &crate::Device,
    create_info: &vk::ImageCreateInfo,
    location: gpu_allocator::MemoryLocation,
) -> Result<(vk::Image, Allocation), anyhow::Error> {
    unsafe {
        let image = device.raw().create_image(create_info, None)?;
        let requirements = device.raw().get_image_memory_requirements(image);
        let allocation = device.allocate_memory(AllocationCreateDesc {
            name: "image",
            requirements,
            location,
            linear: false,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        device
            .raw()
            .bind_image_memory(image, allocation.memory(), allocation.offset())?;

        Ok((image, allocation))
    }
}
pub fn delete_image(
    device: &crate::Device,
    image: vk::Image,
    allocation: Allocation,
) -> Result<(), AllocationError> {
    unsafe {
        device.raw().destroy_image(image, None);
    }
    device.free_memory(allocation)
}