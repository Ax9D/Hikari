use std::{collections::HashSet, ffi::CStr, io::Read, sync::Arc};

use ash::{
    extensions::khr::{Surface, Swapchain},
    prelude::VkResult,
    vk::{self},
};
use gpu_allocator::vulkan::*;
use parking_lot::{Mutex, MutexGuard};

use crate::descriptor::DescriptorSetLayoutCache;

const N_DEVICE_FEATURES: usize =
    std::mem::size_of::<vk::PhysicalDeviceFeatures>() / std::mem::size_of::<u32>();
struct PhysicalDeviceInfo {
    pub device: vk::PhysicalDevice,
    pub graphics_queue: Option<u32>,
    pub present_queue: Option<u32>,
    pub compute_queue: Option<u32>,
    pub swapchain_support_details: SwapchainSupportDetails,
    pub properties: vk::PhysicalDeviceProperties,
    pub suitable: bool,
    pub features: [bool; N_DEVICE_FEATURES],
}
impl PhysicalDeviceInfo {
    pub fn process_device(
        device: vk::PhysicalDevice,
        instance: &ash::Instance,
        surface: &vk::SurfaceKHR,
        surface_loader: &Surface,
        required_extensions: &[&CStr],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let graphics_queue = Self::get_graphics_queue(&device, instance);
        let present_queue = Self::get_present_queue(&device, instance, surface, surface_loader);
        let compute_queue = Self::get_compute_queue(&device, instance);

        let properties = unsafe { instance.get_physical_device_properties(device) };

        let is_discrete_gpu = properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU;

        let extensions = unsafe { instance.enumerate_device_extension_properties(device) }?;

        let extensions_support = required_extensions.iter().all(|x| {
            extensions
                .iter()
                .find(|&&y| {
                    let weirdness = unsafe { &*{ x.to_bytes() as *const [u8] as *const [i8] } };

                    let other = &y.extension_name[..weirdness.len()];

                    other.eq(weirdness)
                })
                .is_some()
        });

        let swapchain_support_details =
            SwapchainSupportDetails::create(&device, surface, surface_loader)?;

        log::info!(
            "\ngraphics_queue: {}\npresent_queue: {}\nis_discrete_gpu: {}\nextension_support : {}",
            graphics_queue.is_some(),
            present_queue.is_some(),
            is_discrete_gpu,
            extensions_support
        );

        let suitable = graphics_queue.is_some()
            && present_queue.is_some()
            && compute_queue.is_some()
            && extensions_support;

        let vk_features = unsafe { instance.get_physical_device_features(device) };
        let mut features = [false; N_DEVICE_FEATURES];

        features[DeviceFeature::SamplerAnisotropy as usize] = vk_features.sampler_anisotropy == 1;
        features[DeviceFeature::GeometryShader as usize] = vk_features.geometry_shader == 1;
        features[DeviceFeature::TesselationShader as usize] = vk_features.tessellation_shader == 1;
        features[DeviceFeature::ShaderFloat64 as usize] = vk_features.shader_float64 == 1;

        Ok(Self {
            device,
            properties,
            graphics_queue,
            present_queue,
            compute_queue,
            suitable,
            swapchain_support_details,
            features,
        })
    }
    fn get_graphics_queue(device: &vk::PhysicalDevice, instance: &ash::Instance) -> Option<u32> {
        let props = unsafe { instance.get_physical_device_queue_family_properties(*device) };

        props
            .iter()
            .enumerate()
            .filter_map(|(index, prop)| {
                log::debug!("{:?}", prop.queue_flags);
                if prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    Some(index as u32)
                } else {
                    None
                }
            })
            .next()
    }
    fn get_compute_queue(device: &vk::PhysicalDevice, instance: &ash::Instance) -> Option<u32> {
        let props = unsafe { instance.get_physical_device_queue_family_properties(*device) };

        props
            .iter()
            .enumerate()
            .filter_map(|(index, prop)| {
                if prop.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                    Some(index as u32)
                } else {
                    None
                }
            })
            .next()
    }
    fn get_present_queue(
        device: &vk::PhysicalDevice,
        instance: &ash::Instance,
        surface: &vk::SurfaceKHR,
        surface_loader: &Surface,
    ) -> Option<u32> {
        let props = unsafe { instance.get_physical_device_queue_family_properties(*device) };

        props
            .iter()
            .enumerate()
            .filter_map(|(index, prop)| unsafe {
                match surface_loader
                    .get_physical_device_surface_support(*device, index as u32, *surface)
                    .unwrap()
                {
                    true => Some(index as u32),
                    false => None,
                }
            })
            .next()
    }
    fn to_physical_device(self) -> Option<PhysicalDevice> {
        if self.suitable {
            let mut unique_queues = HashSet::new();
            let graphics = self.graphics_queue.unwrap();
            let compute = self.compute_queue.unwrap();

            unique_queues.insert(graphics);
            unique_queues.insert(compute);

            Some(PhysicalDevice {
                graphics_queue_ix: graphics,
                present_queue_ix: self.present_queue.unwrap(),
                compute_queue_ix: compute,
                device: self.device,
                swapchain_suport_details: self.swapchain_support_details,
                vk_props: self.properties,
                features: self.features,
                props: self.properties.into(),
                unique_queue_ixs: unique_queues.drain().collect(),
            })
        } else {
            None
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
    ) -> Result<Self, Box<dyn std::error::Error>> {
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

pub enum DeviceFeature {
    SamplerAnisotropy = 0,
    GeometryShader,
    TesselationShader,
    ShaderFloat64,
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
pub(crate) struct PhysicalDevice {
    device: vk::PhysicalDevice,
    graphics_queue_ix: u32,
    present_queue_ix: u32,
    compute_queue_ix: u32,
    unique_queue_ixs: Vec<u32>,
    swapchain_suport_details: SwapchainSupportDetails,
    vk_props: vk::PhysicalDeviceProperties,
    features: [bool; N_DEVICE_FEATURES],

    props: PhysicalDeviceProperties,
}

impl PhysicalDevice {
    pub fn pick_optimal(
        entry: &ash::Entry,
        instance: &ash::Instance,
        surface: &vk::SurfaceKHR,
        surface_loader: &Surface,
        required_extensions: &[&CStr],
    ) -> Result<PhysicalDevice, Box<dyn std::error::Error>> {
        let devices = unsafe { instance.enumerate_physical_devices()? };

        for device in devices {
            let device_info = PhysicalDeviceInfo::process_device(
                device,
                instance,
                &surface,
                &surface_loader,
                &required_extensions,
            )?;
            let device = device_info.to_physical_device();
            if let Some(device) = device {
                return Ok(device);
            }
        }

        Err("Failed to find suitable physical device".into())
    }
    pub fn raw(&self) -> vk::PhysicalDevice {
        self.device
    }
    pub fn is_supported(&self, feature: DeviceFeature) -> bool {
        self.features[feature as usize]
    }
    pub fn graphics_queue_index(&self) -> u32 {
        self.graphics_queue_ix
    }
    pub fn present_queue_index(&self) -> u32 {
        self.present_queue_ix
    }
    pub fn compute_queue_ix(&self) -> u32 {
        self.compute_queue_ix
    }
    pub fn unique_queue_indices(&self) -> &[u32] {
        &self.unique_queue_ixs
    }

    pub fn swapchain_support_details(&self) -> &SwapchainSupportDetails {
        &self.swapchain_suport_details
    }
    pub fn properties(&self) -> &PhysicalDeviceProperties {
        &self.props
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
        let required_extensions = [Swapchain::name()];
        let physical_device = PhysicalDevice::pick_optimal(
            entry,
            &instance,
            surface,
            surface_loader,
            &required_extensions,
        )?;

        log::debug!("Picked physical device");
        log::info!("{}", physical_device.properties().name());

        const QUEUE_PRIORITIES: [f32; 1] = [1.0];
        let mut unique_queue = HashSet::new();
        unique_queue.insert(physical_device.graphics_queue_index());
        unique_queue.insert(physical_device.present_queue_index());

        let queue_create_infos: Vec<_> = unique_queue
            .iter()
            .map(|&queue_ix| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(queue_ix as u32)
                    .queue_priorities(&QUEUE_PRIORITIES)
                    .build()
            })
            .collect();

        let required_extensions = &required_extensions
            .iter()
            .map(|&x| x.as_ptr())
            .collect::<Vec<_>>();

        let enabled_features =
            unsafe { instance.get_physical_device_features(physical_device.raw()) };

        let device_create_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(required_extensions)
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&enabled_features)
            .build();

        let device =
            unsafe { instance.create_device(physical_device.raw(), &device_create_info, None) }?;

        log::debug!("Created logical device");

        let ash_device = device;

        let memory_allocator = Mutex::new(Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: ash_device.clone(),
            physical_device: physical_device.raw(),
            debug_settings: Default::default(),
            buffer_device_address: false,
        })?);

        memory_allocator.lock().report_memory_leaks(log::Level::Debug);

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
            pipeline_cache,
            memory_allocator,
            descriptor_set_layout_cache,

            shader_compiler,
            extensions,
        }))
    }
    pub fn vendor(&self) -> &str {
        self.physical_device.properties().vendor_string()
    }

    pub fn model(&self) -> &str {
        self.physical_device.properties().name()
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


        log::debug!("Read {} bytes from pipeline cache, {}", data.len(), VK_PIPELINE_CACHE_FILE);
        data
    }
    fn write_pipeline_cache_to_disk(&self) -> VkResult<()>{
        let data = unsafe { self.raw().get_pipeline_cache_data(self.pipeline_cache())? };

        log::debug!("Writing {} bytes to pipeline cache, {}", data.len(), VK_PIPELINE_CACHE_FILE);

        std::fs::write(VK_PIPELINE_CACHE_FILE, data)
        .expect("Couldn't write to pipeline cache file");
        
        Ok(())
    }
    fn create_pipeline_cache(device: &ash::Device) -> VkResult<vk::PipelineCache> {
        let pipeline_cache_data = Self::read_pipeline_cache_from_disk();
        let create_info = vk::PipelineCacheCreateInfo::builder()
        .initial_data(&pipeline_cache_data);

        let cache_from_previous_data = unsafe { device.create_pipeline_cache(&create_info, None) };
        match  cache_from_previous_data {
            Ok(cache) => Ok(cache),

            //If it errors try to create an empty cache
            Err(_) => {
                log::debug!("Pipeline cache from disk seems to be unusable, creating empty cache");

                let create_info = vk::PipelineCacheCreateInfo::builder()
                .initial_data(&[]);

                unsafe { device.create_pipeline_cache(&create_info, None) }
            },
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
        unsafe {
            self.raw()
                .get_device_queue(self.physical_device().graphics_queue_index(), 0)
        }
    }
    pub(crate) unsafe fn submit_commands_immediate(
        &self,
        func: impl FnOnce(vk::CommandBuffer) -> VkResult<()>,
    ) -> VkResult<()> {
        let device = self.raw();

        let fence = device.create_fence(&vk::FenceCreateInfo::builder(), None)?;

        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(self.physical_device().graphics_queue_index())
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

    #[deprecated]
    pub fn is_anisotropy_supported(&self) -> bool {
        self.physical_device
            .is_supported(DeviceFeature::SamplerAnisotropy)
    }
}
impl Drop for Device {
    fn drop(&mut self) {
        
        self.write_pipeline_cache_to_disk();

        log::debug!("Dropped Device");
    }
}
