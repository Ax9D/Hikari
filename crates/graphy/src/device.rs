use std::{
    collections::HashSet,
    ffi::{CStr, CString},
    io::Read,
    sync::Arc,
};

use ash::{
    extensions::khr::{Surface, Swapchain},
    prelude::VkResult,
    vk::{self, QueueFamilyProperties},
};
use gpu_allocator::vulkan::*;
use parking_lot::{Mutex, MutexGuard};

use crate::descriptor::DescriptorSetLayoutCache;

pub struct PhysicalDevice {
    pub raw: vk::PhysicalDevice,
    pub queue_families: Vec<QueueFamilyProperties>,
    pub(crate) swapchain_support_details: SwapchainSupportDetails,
    pub properties: vk::PhysicalDeviceProperties,
    pub extensions: Vec<CString>,
    pub mem_properties: vk::PhysicalDeviceMemoryProperties,
    pub features: vk::PhysicalDeviceFeatures2,
}
impl PhysicalDevice {
    pub fn enumerate(
        instance: &ash::Instance,
        surface: &vk::SurfaceKHR,
        surface_loader: &Surface,
    ) -> VkResult<Vec<PhysicalDevice>> {
        let raw_devices = unsafe { instance.enumerate_physical_devices() }?;

        let mut devices = Vec::new();
        for device in raw_devices {
            devices.push(Self::process_device(
                device,
                instance,
                surface,
                surface_loader,
            )?)
        }

        Ok(devices)
    }
    fn process_device(
        device: vk::PhysicalDevice,
        instance: &ash::Instance,
        surface: &vk::SurfaceKHR,
        surface_loader: &Surface,
    ) -> VkResult<Self> {
        let properties = unsafe { instance.get_physical_device_properties(device) };
        let mem_properties = unsafe { instance.get_physical_device_memory_properties(device) };

        let is_discrete_gpu = properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU;

        let extensions = unsafe { instance.enumerate_device_extension_properties(device) }?;

        let extensions = extensions
            .iter()
            .map(|extension| {
                let extension_cstr = unsafe {
                    std::ffi::CStr::from_ptr(
                        extension.extension_name.as_ptr() as *const std::os::raw::c_char
                    )
                };
                extension_cstr.to_owned()
            })
            .collect();

        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(device) };
        let swapchain_support_details =
            SwapchainSupportDetails::create(&device, surface, surface_loader)?;

        let vk_features = unsafe { instance.get_physical_device_features(device) };

        let mut vk_features2 = vk::PhysicalDeviceFeatures2::default();
        unsafe {
            instance.get_physical_device_features2(device, &mut vk_features2);
        }

        Ok(Self {
            raw: device,
            properties,
            queue_families,
            swapchain_support_details,
            features: vk_features2,
            mem_properties,
            extensions,
        })
    }

    pub fn get_present_queue(
        &self,
        instance: &ash::Instance,
        surface: &vk::SurfaceKHR,
        surface_loader: &Surface,
    ) -> Option<u32> {
        self.queue_families
            .iter()
            .enumerate()
            .filter_map(|(index, prop)| unsafe {
                match surface_loader
                    .get_physical_device_surface_support(self.raw, index as u32, *surface)
                    .unwrap()
                {
                    true => Some(index as u32),
                    false => None,
                }
            })
            .next()
    }

    pub fn get_unified_queue(&self) -> Option<u32> {
        use vk::QueueFlags as qf;
        self.queue_families
            .iter()
            .enumerate()
            .find(|(_, props)| {
                props
                    .queue_flags
                    .contains(qf::GRAPHICS | qf::COMPUTE | qf::TRANSFER)
            })
            .map(|(ix, _)| ix as u32)
    }

    fn get_properties(&self) -> PhysicalDeviceProperties {
        let props = &self.properties;
        let name = String::from_utf8(props.device_name.iter().map(|&x| x as u8).collect()).unwrap();
        let vendor_id = props.vendor_id;
        let driver_version = props.driver_version;

        PhysicalDeviceProperties {
            name,
            vendor_id,
            driver_version,
        }
    }
}

pub(crate) struct SwapchainSupportDetails {
    pub(crate) formats: Vec<vk::SurfaceFormatKHR>,
    pub(crate) capabilities: vk::SurfaceCapabilitiesKHR,
    pub(crate) present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupportDetails {
    pub fn create(
        device: &vk::PhysicalDevice,
        surface: &vk::SurfaceKHR,
        surface_loader: &Surface,
    ) -> VkResult<Self> {
        let formats =
            unsafe { surface_loader.get_physical_device_surface_formats(*device, *surface) }?;
        let capabilities =
            unsafe { surface_loader.get_physical_device_surface_capabilities(*device, *surface) }?;
        let present_modes =
            unsafe { surface_loader.get_physical_device_surface_present_modes(*device, *surface) }?;

        Ok(Self {
            capabilities,
            formats,
            present_modes,
        })
    }
}

#[derive(Clone)]
pub struct PhysicalDeviceProperties {
    name: String,
    vendor_id: u32,
    driver_version: u32,
}
impl PhysicalDeviceProperties {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn vendor_string(&self) -> &'static str {
        match self.vendor_id {
            0x1002 => "AMD",
            0x10DE => "Nvidia",
            0x8086 => "Intel",
            0x13B5 => "ARM",
            _ => "Unknown",
        }
    }
}
impl From<vk::PhysicalDeviceProperties> for PhysicalDeviceProperties {
    fn from(props: vk::PhysicalDeviceProperties) -> Self {
        let name = String::from_utf8(props.device_name.iter().map(|&x| x as u8).collect()).unwrap();
        let vendor_id = props.vendor_id;
        let driver_version = props.driver_version;

        Self {
            name,
            vendor_id,
            driver_version,
        }
    }
}
struct RawDevice {
    inner: ash::Device,
    instance: ash::Instance,
}
impl Drop for RawDevice {
    fn drop(&mut self) {
        unsafe {
            //self.instance.destroy_instance(None);
            //self.inner.destroy_device(None);
            log::info!("Dropped vkDevice");
            // log::info!("Dropped vkInstance");
        }
    }
}

pub struct VkExtensions {
    pub synchronization2: ash::extensions::khr::Synchronization2,
}

const VK_PIPELINE_CACHE_FILE: &'static str = "vk_pipeline_cache";

pub struct Device {
    physical_device: PhysicalDevice,
    device_properties: PhysicalDeviceProperties,
    pub(crate) unified_queue_ix: u32,
    pub(crate) present_queue_ix: u32,

    shader_compiler: Mutex<shaderc::Compiler>,
    memory_allocator: Mutex<gpu_allocator::vulkan::Allocator>,
    descriptor_set_layout_cache: Mutex<DescriptorSetLayoutCache>,
    extensions: VkExtensions,
    pipeline_cache: vk::PipelineCache,

    raw_device: RawDevice,
}

impl Device {
    pub(crate) fn create(
        entry: &ash::Entry,
        instance: ash::Instance,
        surface: &vk::SurfaceKHR,
        surface_loader: &Surface,
    ) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let required_extensions = [Swapchain::name(), vk::KhrSynchronization2Fn::name()];

        let physical_device = Self::pick_optimal(
            entry,
            &instance,
            surface,
            surface_loader,
            &required_extensions,
        )
        .ok_or("Failed to find suitable physical device")?;

        let props = physical_device.get_properties();
        log::debug!("Picked physical device");
        log::info!("{}", props.name());

        const QUEUE_PRIORITIES: [f32; 1] = [1.0];

        let unified_queue_ix = physical_device.get_unified_queue().unwrap();
        let present_queue_ix = physical_device.get_unified_queue().unwrap();

        let queue_create_infos = [*vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(unified_queue_ix)
            .queue_priorities(&QUEUE_PRIORITIES)];

        let required_extensions = &required_extensions
            .iter()
            .map(|&x| x.as_ptr())
            .collect::<Vec<_>>();

        let enabled_features = vk::PhysicalDeviceFeatures {
            sampler_anisotropy: 1,

            ..Default::default()
        };

        let mut sync2 =
            vk::PhysicalDeviceSynchronization2FeaturesKHR::builder().synchronization2(true);
        let device_create_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(required_extensions)
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&enabled_features)
            .push_next(&mut sync2)
            .build();

        let device =
            unsafe { instance.create_device(physical_device.raw, &device_create_info, None) }?;

        log::debug!("Created logical device");

        let ash_device = device;

        let memory_allocator = Mutex::new(Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: ash_device.clone(),
            physical_device: physical_device.raw,
            debug_settings: Default::default(),
            buffer_device_address: false,
        })?);

        memory_allocator
            .lock()
            .report_memory_leaks(log::Level::Debug);

        let shader_compiler = Mutex::new(
            shaderc::Compiler::new().ok_or("Failed to initialize shaderc compiler".to_owned())?,
        );

        //let frame_res = FrameResources::new(device.inner.clone(), &physical_device)?;

        // let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
        // .queue_family_index(queue_family_index)

        let extensions = Self::setup_extension(&instance, &ash_device);

        let pipeline_cache = Self::create_pipeline_cache(&ash_device)?;

        let descriptor_set_layout_cache = Mutex::new(DescriptorSetLayoutCache::new(&ash_device));
        let raw_device = RawDevice {
            inner: ash_device,
            instance: instance,
        };

        //let device = RawDevice { inner: ash_device.clone() };
        Ok(Arc::new(Self {
            raw_device,
            physical_device,
            device_properties: props,

            unified_queue_ix,
            present_queue_ix,

            pipeline_cache,
            memory_allocator,
            descriptor_set_layout_cache,

            shader_compiler,
            extensions,
        }))
    }
    fn pick_optimal(
        entry: &ash::Entry,
        instance: &ash::Instance,
        surface: &vk::SurfaceKHR,
        surface_loader: &Surface,
        required_extensions: &[&'static CStr],
    ) -> Option<PhysicalDevice> {
        let physical_devices = PhysicalDevice::enumerate(instance, surface, surface_loader).ok()?;
        for device in physical_devices {
            let present_support = device
                .get_present_queue(instance, surface, surface_loader)
                .is_some();

            let unified_queue = device.get_unified_queue().is_some();

            if present_support && unified_queue {
                return Some(device);
            }
        }

        None
    }
    pub fn vendor(&self) -> &str {
        self.device_properties.vendor_string()
    }

    pub fn model(&self) -> &str {
        self.device_properties.name()
    }
    fn setup_extension(instance: &ash::Instance, device: &ash::Device) -> VkExtensions {
        let synchronization2 = ash::extensions::khr::Synchronization2::new(instance, device);

        VkExtensions { synchronization2 }
    }
    pub fn extensions(&self) -> &VkExtensions {
        &self.extensions
    }
    fn read_pipeline_cache_from_disk() -> Vec<u8> {
        let mut data = Vec::new();

        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(VK_PIPELINE_CACHE_FILE)
            .expect("Couldn't create pipeline cache file")
            .read_to_end(&mut data);

        log::debug!(
            "Read {} bytes from pipeline cache, {}",
            data.len(),
            VK_PIPELINE_CACHE_FILE
        );
        data
    }
    fn write_pipeline_cache_to_disk(&self) -> VkResult<()> {
        let data = unsafe { self.raw().get_pipeline_cache_data(self.pipeline_cache())? };

        log::debug!(
            "Writing {} bytes to pipeline cache, {}",
            data.len(),
            VK_PIPELINE_CACHE_FILE
        );

        std::fs::write(VK_PIPELINE_CACHE_FILE, data)
            .expect("Couldn't write to pipeline cache file");

        Ok(())
    }
    fn create_pipeline_cache(device: &ash::Device) -> VkResult<vk::PipelineCache> {
        let pipeline_cache_data = Self::read_pipeline_cache_from_disk();
        let create_info = vk::PipelineCacheCreateInfo::builder().initial_data(&pipeline_cache_data);

        let cache_from_previous_data = unsafe { device.create_pipeline_cache(&create_info, None) };
        match cache_from_previous_data {
            Ok(cache) => Ok(cache),

            //If it errors try to create an empty cache
            Err(_) => {
                log::debug!("Pipeline cache from disk seems to be unusable, creating empty cache");

                let create_info = vk::PipelineCacheCreateInfo::builder().initial_data(&[]);

                unsafe { device.create_pipeline_cache(&create_info, None) }
            }
        }
    }
    pub(crate) fn pipeline_cache(&self) -> vk::PipelineCache {
        self.pipeline_cache
    }
    pub fn raw(&self) -> &ash::Device {
        &self.raw_device.inner
    }
    pub fn instance(&self) -> &ash::Instance {
        &self.raw_device.instance
    }
    pub(crate) fn shader_compiler(&self) -> MutexGuard<'_, shaderc::Compiler> {
        self.shader_compiler.lock()
    }
    pub(crate) fn set_layout_cache(&self) -> MutexGuard<'_, DescriptorSetLayoutCache> {
        self.descriptor_set_layout_cache.lock()
    }

    pub(crate) fn physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }
    pub(crate) fn allocate_memory(
        &self,
        desc: AllocationCreateDesc,
    ) -> gpu_allocator::Result<Allocation> {
        self.memory_allocator.lock().allocate(&desc)
    }
    pub(crate) fn free_memory(&self, allocation: Allocation) -> gpu_allocator::Result<()> {
        self.memory_allocator.lock().free(allocation)
    }

    pub(crate) fn graphics_queue(&self) -> vk::Queue {
        unsafe { self.raw().get_device_queue(self.unified_queue_ix, 0) }
    }
    pub(crate) fn present_queue(&self) -> vk::Queue {
        unsafe { self.raw().get_device_queue(self.present_queue_ix, 0) }
    }
    pub(crate) unsafe fn submit_commands_immediate(
        &self,
        func: impl FnOnce(vk::CommandBuffer) -> VkResult<()>,
    ) -> VkResult<()> {
        let device = self.raw();

        let fence = device.create_fence(&vk::FenceCreateInfo::builder(), None)?;

        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(self.unified_queue_ix)
            .flags(vk::CommandPoolCreateFlags::TRANSIENT);

        let now = std::time::Instant::now();

        let cmd_pool = unsafe { device.create_command_pool(&create_info, None) }?;

        println!("Command pool creation took: {:?}", now.elapsed());

        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(cmd_pool)
            .command_buffer_count(1);

        let cmd = self.raw().allocate_command_buffers(&create_info)?[0];

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device.begin_command_buffer(cmd, &begin_info)?;

        (func)(cmd)?;

        device.end_command_buffer(cmd)?;

        let cmd = &[cmd];
        let submit_info = vk::SubmitInfo::builder().command_buffers(cmd);

        let now = std::time::Instant::now();

        device.queue_submit(self.graphics_queue(), &[*submit_info], fence)?;

        device.wait_for_fences(&[fence], true, 999999999)?;

        device.destroy_fence(fence, None);

        device.reset_command_pool(cmd_pool, vk::CommandPoolResetFlags::empty())?;

        self.raw().destroy_command_pool(cmd_pool, None);

        println!("Submitted commands, took: {:?}", now.elapsed());
        Ok(())
    }

    pub fn vk_features(&self) -> &vk::PhysicalDeviceFeatures {
        &self.physical_device.features.features
    }
}
impl Drop for Device {
    fn drop(&mut self) {
        self.write_pipeline_cache_to_disk().unwrap();

        log::debug!("Dropped Device");
    }
}
