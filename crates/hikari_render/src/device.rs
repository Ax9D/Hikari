use std::{
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

use crate::{descriptor::DescriptorSetLayoutCache, swapchain::SurfaceData};

const VK_PIPELINE_CACHE_FILE: &str = "vk_pipeline_cache";

pub struct PhysicalDevice {
    pub raw: vk::PhysicalDevice,
    pub queue_families: Vec<QueueFamilyProperties>,
    pub properties: vk::PhysicalDeviceProperties,
    pub extensions: Vec<CString>,
    pub mem_properties: vk::PhysicalDeviceMemoryProperties,
    pub vk_features: vk::PhysicalDeviceFeatures2,
    pub features: Features,
}
impl PhysicalDevice {
    pub fn enumerate(instance: &ash::Instance) -> VkResult<Vec<PhysicalDevice>> {
        let raw_devices = unsafe { instance.enumerate_physical_devices() }?;

        let mut devices = Vec::new();
        for device in raw_devices {
            devices.push(Self::process_device(device, instance)?)
        }

        Ok(devices)
    }
    fn process_device(device: vk::PhysicalDevice, instance: &ash::Instance) -> VkResult<Self> {
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

        let vk_features = unsafe { instance.get_physical_device_features(device) };

        let mut vk_features2 = vk::PhysicalDeviceFeatures2::default();
        unsafe {
            instance.get_physical_device_features2(device, &mut vk_features2);
        }

        let features = vk_features2.features.into();

        Ok(Self {
            raw: device,
            properties,
            queue_families,
            vk_features: vk_features2,
            features,
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

    pub(crate) fn get_swapchain_support_details(
        &self,
        surface: &vk::SurfaceKHR,
        surface_loader: &Surface,
    ) -> VkResult<SwapchainSupportDetails> {
        SwapchainSupportDetails::create(&self.raw, surface, surface_loader)
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

    pub fn get_supported_depth_stencil_format(
        &self,
        instance: &ash::Instance,
        candidates: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Option<vk::Format> {
        for &format in candidates {
            let properties =
                unsafe { instance.get_physical_device_format_properties(self.raw, format) };
            let supported = match tiling {
                vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                _ => unreachable!(),
            };
            if supported {
                return Some(format);
            }
        }

        None
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
        let len = props.device_name.iter().position(|&ch| ch == 0).unwrap();
        let device_name = &props.device_name[0..len];

        let name = String::from_utf8(device_name.iter().map(|&x| x as u8).collect()).unwrap();
        let vendor_id = props.vendor_id;
        let driver_version = props.driver_version;

        Self {
            name,
            vendor_id,
            driver_version,
        }
    }
}
struct Queue {

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

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

use nvidia_aftermath_rs::Aftermath;

/// Represents the GPU device being used
/// Is used to allocate resources on the GPU
pub struct Device {
    physical_device: PhysicalDevice,
    device_properties: PhysicalDeviceProperties,
    pub(crate) unified_queue_ix: u32,
    //pub(crate) present_queue_ix: u32,
    unified_queue: Mutex<vk::Queue>,

    shader_compiler: Mutex<shaderc::Compiler>,

    memory_allocator: Arc<std::sync::Mutex<gpu_allocator::vulkan::Allocator>>, //Using std::sync::Mutex for better compatibility
    descriptor_set_layout_cache: Mutex<DescriptorSetLayoutCache>,
    extensions: VkExtensions,
    pipeline_cache: vk::PipelineCache,
    aftermath: Option<Aftermath>,

    raw_device: RawDevice,
    entry: ash::Entry,
}

impl Device {
    pub(crate) fn create(
        entry: ash::Entry,
        instance: ash::Instance,
        surface_data: Option<&SurfaceData>,
        enable_features: Features,
        debug: bool,
    ) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let mut required_extensions = vec![vk::KhrSynchronization2Fn::name()];

        if surface_data.is_some() {
            required_extensions.push(Swapchain::name());
        }

        let physical_device = Self::pick_optimal(
            &entry,
            &instance,
            surface_data,
            &required_extensions,
            enable_features,
        )
        .ok_or("Failed to find suitable physical device")?;

        let props = PhysicalDeviceProperties::from(physical_device.properties);
        log::debug!("Picked physical device");
        log::info!("{}", props.name());

        const QUEUE_PRIORITIES: [f32; 1] = [1.0];

        let unified_queue_ix = physical_device.get_unified_queue().unwrap();

        let queue_create_infos = [*vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(unified_queue_ix)
            .queue_priorities(&QUEUE_PRIORITIES)];

        let required_extensions = &required_extensions
            .iter()
            .map(|&x| x.as_ptr())
            .collect::<Vec<_>>();

        let enabled_features = enable_features.into();

        let mut sync2 =
            vk::PhysicalDeviceSynchronization2FeaturesKHR::builder().synchronization2(true);

        let mut diag_config_nv = vk::DeviceDiagnosticsConfigCreateInfoNV::builder().flags(
            vk::DeviceDiagnosticsConfigFlagsNV::ENABLE_AUTOMATIC_CHECKPOINTS
                | vk::DeviceDiagnosticsConfigFlagsNV::ENABLE_RESOURCE_TRACKING
                | vk::DeviceDiagnosticsConfigFlagsNV::ENABLE_SHADER_DEBUG_INFO,
        );

        let mut device_create_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(required_extensions)
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&enabled_features)
            //VK_KHR_synchronization2
            .push_next(&mut sync2);

        if debug {
            device_create_info = device_create_info.push_next(&mut diag_config_nv);
        }

        let ash_device =
            unsafe { instance.create_device(physical_device.raw, &device_create_info, None) }?;

        log::debug!("Created logical device");

        let unified_queue = unsafe { ash_device.get_device_queue(unified_queue_ix, 0) };
        //let present_queue = unsafe { ash_device.get_device_queue(present_queue_ix, 0) };

        let memory_allocator = std::sync::Mutex::new(Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: ash_device.clone(),
            physical_device: physical_device.raw,
            debug_settings: gpu_allocator::AllocatorDebugSettings {
                log_leaks_on_shutdown: true,
                ..Default::default()
            },
            buffer_device_address: false,
        })?);

        let memory_allocator = Arc::new(memory_allocator);

        memory_allocator
            .lock()
            .unwrap()
            .report_memory_leaks(log::Level::Debug);

        let shader_compiler = Mutex::new(
            shaderc::Compiler::new()
                .ok_or_else(|| "Failed to initialize shaderc compiler".to_owned())?,
        );

        let extensions = Self::setup_extension(&instance, &ash_device);

        let pipeline_cache = Self::create_pipeline_cache(&ash_device)?;

        let descriptor_set_layout_cache = Mutex::new(DescriptorSetLayoutCache::new(&ash_device));
        let raw_device = RawDevice {
            inner: ash_device,
            instance,
        };

        let aftermath = if debug {
            match Aftermath::initialize() {
                Ok(ok) => Some(ok),
                Err(err) => {
                    log::error!("TODO: Handle Aftermath initialization error!");
                    None
                }
            }
        } else {
            None
        };

        Ok(Arc::new(Self {
            entry,
            raw_device,
            physical_device,
            device_properties: props,

            unified_queue_ix,
            //present_queue_ix,
            unified_queue: Mutex::new(unified_queue),
            //present_queue,

            pipeline_cache,
            memory_allocator,
            descriptor_set_layout_cache,

            shader_compiler,
            extensions,
            aftermath,
        }))
    }
    fn pick_optimal(
        entry: &ash::Entry,
        instance: &ash::Instance,
        surface_data: Option<&SurfaceData>,
        required_extensions: &[&'static CStr],
        required_features: Features,
    ) -> Option<PhysicalDevice> {
        let physical_devices = PhysicalDevice::enumerate(instance).ok()?;
        for device in physical_devices {
            let unified_queue = device.get_unified_queue().is_some();

            if unified_queue && device.features.contains(required_features) {
                if let Some(surface_data) = surface_data {
                    let present_support = device
                        .get_present_queue(
                            instance,
                            &surface_data.surface,
                            &surface_data.surface_loader,
                        )
                        .is_some();

                    if present_support {
                        return Some(device);
                    }

                    return None;
                }
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
        std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(VK_PIPELINE_CACHE_FILE)
            .expect("Couldn't create pipeline cache file")
            .read_to_end(&mut data)
            .expect("Failed to read pipeline cache file");

        log::debug!(
            "Read {} bytes from pipeline cache, {}",
            data.len(),
            VK_PIPELINE_CACHE_FILE
        );
        data
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
    pub fn write_pipeline_cache_to_disk(&self) -> VkResult<()> {
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

    pub fn physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }
    pub fn allocate_memory(&self, desc: AllocationCreateDesc) -> gpu_allocator::Result<Allocation> {
        self.memory_allocator
            .lock()
            .expect("Failed to lock memory allocator")
            .allocate(&desc)
    }
    pub fn free_memory(&self, allocation: Allocation) -> gpu_allocator::Result<()> {
        self.memory_allocator
            .lock()
            .expect("Failed to lock memory allocator")
            .free(allocation)
    }
    pub fn allocator(&self) -> &Arc<std::sync::Mutex<Allocator>> {
        &self.memory_allocator
    }

    pub fn unified_queue(&self) -> MutexGuard<'_, vk::Queue> {
        self.unified_queue.lock()
    }
    // pub fn present_queue(&self) -> vk::Queue {
    //     self.present_queue
    // }
    pub(crate) fn graphics_queue_submit(
        &self,
        submits: &[vk::SubmitInfo],
        fence: vk::Fence,
    ) -> VkResult<()> {
        //log::debug!("vkQueueSubmit");
        let queue = self.unified_queue();
        unsafe {
            self.raw()
                .queue_submit(*queue, submits, fence)
        }
    }
    pub(crate) unsafe fn submit_commands_immediate(
        &self,
        func: impl FnOnce(vk::CommandBuffer) -> VkResult<()>,
    ) -> VkResult<()> {
        let device = self.raw();

        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(self.unified_queue_ix)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let now = std::time::Instant::now();

        let cmd_pool = unsafe { device.create_command_pool(&create_info, None) }?;

        //println!("Command pool creation took: {:?}", now.elapsed());

        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(cmd_pool)
            .command_buffer_count(1);

        let cmd = self.raw().allocate_command_buffers(&create_info)?[0];

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device.begin_command_buffer(cmd, &begin_info)?;

        (func)(cmd)?;

        device.end_command_buffer(cmd)?;

        let cmd = [cmd];
        let submit_info = vk::SubmitInfo::builder().command_buffers(&cmd);

        let now = std::time::Instant::now();

        let fence = device.create_fence(&vk::FenceCreateInfo::builder(), None)?;
        self.graphics_queue_submit(&[*submit_info], fence)?;

        let fences = [fence];
        device.wait_for_fences(&fences, true, 5_000_000_000)?;
        device.reset_fences(&fences)?;
        device.destroy_fence(fence, None);

        //device.reset_command_buffer(cmd[0], vk::CommandBufferResetFlags::empty())?;
        device.reset_command_pool(cmd_pool, vk::CommandPoolResetFlags::empty())?;

        self.raw().destroy_command_pool(cmd_pool, None);

        //println!("Submitted commands, took: {:?}", now.elapsed());
        Ok(())
    }

    pub fn features(&self) -> Features {
        self.physical_device.features
    }
    pub fn is_feature_supported(&self, feature: Features) -> bool {
        self.physical_device.features.contains(feature)
    }

    pub fn supported_depth_stencil_format(&self) -> vk::Format {
        let candidates = [
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
            vk::Format::D16_UNORM_S8_UINT,
        ];
        let optimal_tiling = self.physical_device.get_supported_depth_stencil_format(
            self.instance(),
            &candidates,
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        );
        let linear_tiling = self.physical_device.get_supported_depth_stencil_format(
            self.instance(),
            &candidates,
            vk::ImageTiling::LINEAR,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        );

        optimal_tiling
            .or(linear_tiling)
            .expect("Device doesn't support any depth formats")
    }
    pub fn supported_depth_only_format(&self) -> vk::Format {
        let candidates = [vk::Format::D32_SFLOAT, vk::Format::D16_UNORM];
        let optimal_tiling = self.physical_device.get_supported_depth_stencil_format(
            self.instance(),
            &candidates,
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        );
        let linear_tiling = self.physical_device.get_supported_depth_stencil_format(
            self.instance(),
            &candidates,
            vk::ImageTiling::LINEAR,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        );

        optimal_tiling
            .or(linear_tiling)
            .expect("Device doesn't support any depth formats")
    }

    pub fn wait_for_aftermath_dump(&self) -> Result<(), anyhow::Error> {
        match &self.aftermath {
            Some(aftermath) => {
                aftermath.wait_for_dump();
                Ok(())
            },
            None => {
                Err(anyhow::anyhow!("Failed to generate aftermath dump. Aftermath not initialized! Was Gfx initialized in debug mode?"))
            },
        }
    }
}
impl Drop for Device {
    fn drop(&mut self) {
        self.write_pipeline_cache_to_disk().unwrap();
        unsafe { self.raw().device_wait_idle().unwrap() };
        log::debug!("Dropped Device");
    }
}

bitflags::bitflags! {
    ///Bit flag representing the Features that the Device may support (although not guaranteed).
    /// Maps 1:1 to Vulkans VkPhysicalDeviceFeatures
    pub struct Features: u64 {
        const ROBUST_BUFFER_ACCESS = 1 << 0;
        const FULL_DRAW_INDEX_UINT32 = 1 << 1;
        const IMAGE_CUBE_ARRAY = 1 << 2;
        const INDEPENDENT_BLEND = 1 << 3;
        const GEOMETRY_SHADER = 1 << 4;
        const TESSELLATION_SHADER = 1 << 5;
        const SAMPLE_RATE_SHADING = 1 << 6;
        const DUAL_SRC_BLEND = 1 << 7;
        const LOGIC_OP = 1 << 8;
        const MULTI_DRAW_INDIRECT = 1 << 9;
        const DRAW_INDIRECT_FIRST_INSTANCE = 1 << 10;
        const DEPTH_CLAMP = 1 << 11;
        const DEPTH_BIAS_CLAMP = 1 << 12;
        const FILL_MODE_NON_SOLID = 1 << 13;
        const DEPTH_BOUNDS = 1 << 14;
        const WIDE_LINES = 1 << 15;
        const LARGE_POINTS = 1 << 16;
        const ALPHA_TO_ONE = 1 << 17;
        const MULTI_VIEWPORT = 1 << 18;
        const SAMPLER_ANISOTROPY = 1 << 19;
        const TEXTURE_COMPRESSION_ETC2 = 1 << 20;
        const TEXTURE_COMPRESSION_ASTC_LDR = 1 << 21;
        const TEXTURE_COMPRESSION_BC = 1 << 22;
        const OCCLUSION_QUERY_PRECISE = 1 << 23;
        const PIPELINE_STATISTICS_QUERY = 1 << 24;
        const VERTEX_PIPELINE_STORES_AND_ATOMICS = 1 << 25;
        const FRAGMENT_STORES_AND_ATOMICS = 1 << 26;
        const SHADER_TESSELLATION_AND_GEOMETRY_POINT_SIZE = 1 << 27;
        const SHADER_IMAGE_GATHER_EXTENDED = 1 << 28;
        const SHADER_STORAGE_IMAGE_EXTENDED_FORMATS = 1 << 29;
        const SHADER_STORAGE_IMAGE_MULTISAMPLE = 1 << 30;
        const SHADER_STORAGE_IMAGE_READ_WITHOUT_FORMAT = 1 << 31;
        const SHADER_STORAGE_IMAGE_WRITE_WITHOUT_FORMAT = 1 << 32;
        const SHADER_UNIFORM_BUFFER_ARRAY_DYNAMIC_INDEXING = 1 << 33;
        const SHADER_SAMPLED_IMAGE_ARRAY_DYNAMIC_INDEXING = 1 << 34;
        const SHADER_STORAGE_BUFFER_ARRAY_DYNAMIC_INDEXING = 1 << 35;
        const SHADER_STORAGE_IMAGE_ARRAY_DYNAMIC_INDEXING = 1 << 36;
        const SHADER_CLIP_DISTANCE = 1 << 37;
        const SHADER_CULL_DISTANCE = 1 << 38;
        const SHADER_FLOAT64 = 1 << 39;
        const SHADER_INT64 = 1 << 40;
        const SHADER_INT16 = 1 << 41;
        const SHADER_RESOURCE_RESIDENCY = 1 << 42;
        const SHADER_RESOURCE_MIN_LOD = 1 << 43;
        const SPARSE_BINDING = 1 << 44;
        const SPARSE_RESIDENCY_BUFFER = 1 << 45;
        const SPARSE_RESIDENCY_IMAGE_2D = 1 << 46;
        const SPARSE_RESIDENCY_IMAGE_3D = 1 << 47;
        const SPARSE_RESIDENCY_2_SAMPLES = 1 << 48;
        const SPARSE_RESIDENCY_4_SAMPLES = 1 << 49;
        const SPARSE_RESIDENCY_8_SAMPLES = 1 << 50;
        const SPARSE_RESIDENCY_16_SAMPLES = 1 << 51;
        const SPARSE_RESIDENCY_ALIASED = 1 << 52;
        const VARIABLE_MULTISAMPLE_RATE = 1 << 53;
        const INHERITED_QUERIES = 1 << 54;

    }
}

impl Default for Features {
    fn default() -> Self {
        Features::SAMPLER_ANISOTROPY | Features::FILL_MODE_NON_SOLID
    }
}
impl From<vk::PhysicalDeviceFeatures> for Features {
    fn from(vk_features: vk::PhysicalDeviceFeatures) -> Self {
        let mut features = Features::empty();

        if vk_features.robust_buffer_access == 1 {
            features |= Features::ROBUST_BUFFER_ACCESS
        };
        if vk_features.full_draw_index_uint32 == 1 {
            features |= Features::FULL_DRAW_INDEX_UINT32
        };
        if vk_features.image_cube_array == 1 {
            features |= Features::IMAGE_CUBE_ARRAY
        };
        if vk_features.independent_blend == 1 {
            features |= Features::INDEPENDENT_BLEND
        };
        if vk_features.geometry_shader == 1 {
            features |= Features::GEOMETRY_SHADER
        };
        if vk_features.tessellation_shader == 1 {
            features |= Features::TESSELLATION_SHADER
        };
        if vk_features.sample_rate_shading == 1 {
            features |= Features::SAMPLE_RATE_SHADING
        };
        if vk_features.dual_src_blend == 1 {
            features |= Features::DUAL_SRC_BLEND
        };
        if vk_features.logic_op == 1 {
            features |= Features::LOGIC_OP
        };
        if vk_features.multi_draw_indirect == 1 {
            features |= Features::MULTI_DRAW_INDIRECT
        };
        if vk_features.draw_indirect_first_instance == 1 {
            features |= Features::DRAW_INDIRECT_FIRST_INSTANCE
        };
        if vk_features.depth_clamp == 1 {
            features |= Features::DEPTH_CLAMP
        };
        if vk_features.depth_bias_clamp == 1 {
            features |= Features::DEPTH_BIAS_CLAMP
        };
        if vk_features.fill_mode_non_solid == 1 {
            features |= Features::FILL_MODE_NON_SOLID
        };
        if vk_features.depth_bounds == 1 {
            features |= Features::DEPTH_BOUNDS
        };
        if vk_features.wide_lines == 1 {
            features |= Features::WIDE_LINES
        };
        if vk_features.large_points == 1 {
            features |= Features::LARGE_POINTS
        };
        if vk_features.alpha_to_one == 1 {
            features |= Features::ALPHA_TO_ONE
        };
        if vk_features.multi_viewport == 1 {
            features |= Features::MULTI_VIEWPORT
        };
        if vk_features.sampler_anisotropy == 1 {
            features |= Features::SAMPLER_ANISOTROPY
        };
        if vk_features.texture_compression_etc2 == 1 {
            features |= Features::TEXTURE_COMPRESSION_ETC2
        };
        if vk_features.texture_compression_astc_ldr == 1 {
            features |= Features::TEXTURE_COMPRESSION_ASTC_LDR
        };
        if vk_features.texture_compression_bc == 1 {
            features |= Features::TEXTURE_COMPRESSION_BC
        };
        if vk_features.occlusion_query_precise == 1 {
            features |= Features::OCCLUSION_QUERY_PRECISE
        };
        if vk_features.pipeline_statistics_query == 1 {
            features |= Features::PIPELINE_STATISTICS_QUERY
        };
        if vk_features.vertex_pipeline_stores_and_atomics == 1 {
            features |= Features::VERTEX_PIPELINE_STORES_AND_ATOMICS
        };
        if vk_features.fragment_stores_and_atomics == 1 {
            features |= Features::FRAGMENT_STORES_AND_ATOMICS
        };
        if vk_features.shader_tessellation_and_geometry_point_size == 1 {
            features |= Features::SHADER_TESSELLATION_AND_GEOMETRY_POINT_SIZE
        };
        if vk_features.shader_image_gather_extended == 1 {
            features |= Features::SHADER_IMAGE_GATHER_EXTENDED
        };
        if vk_features.shader_storage_image_extended_formats == 1 {
            features |= Features::SHADER_STORAGE_IMAGE_EXTENDED_FORMATS
        };
        if vk_features.shader_storage_image_multisample == 1 {
            features |= Features::SHADER_STORAGE_IMAGE_MULTISAMPLE
        };
        if vk_features.shader_storage_image_read_without_format == 1 {
            features |= Features::SHADER_STORAGE_IMAGE_READ_WITHOUT_FORMAT
        };
        if vk_features.shader_storage_image_write_without_format == 1 {
            features |= Features::SHADER_STORAGE_IMAGE_WRITE_WITHOUT_FORMAT
        };
        if vk_features.shader_uniform_buffer_array_dynamic_indexing == 1 {
            features |= Features::SHADER_UNIFORM_BUFFER_ARRAY_DYNAMIC_INDEXING
        };
        if vk_features.shader_sampled_image_array_dynamic_indexing == 1 {
            features |= Features::SHADER_SAMPLED_IMAGE_ARRAY_DYNAMIC_INDEXING
        };
        if vk_features.shader_storage_buffer_array_dynamic_indexing == 1 {
            features |= Features::SHADER_STORAGE_BUFFER_ARRAY_DYNAMIC_INDEXING
        };
        if vk_features.shader_storage_image_array_dynamic_indexing == 1 {
            features |= Features::SHADER_STORAGE_IMAGE_ARRAY_DYNAMIC_INDEXING
        };
        if vk_features.shader_clip_distance == 1 {
            features |= Features::SHADER_CLIP_DISTANCE
        };
        if vk_features.shader_cull_distance == 1 {
            features |= Features::SHADER_CULL_DISTANCE
        };
        if vk_features.shader_float64 == 1 {
            features |= Features::SHADER_FLOAT64
        };
        if vk_features.shader_int64 == 1 {
            features |= Features::SHADER_INT64
        };
        if vk_features.shader_int16 == 1 {
            features |= Features::SHADER_INT16
        };
        if vk_features.shader_resource_residency == 1 {
            features |= Features::SHADER_RESOURCE_RESIDENCY
        };
        if vk_features.shader_resource_min_lod == 1 {
            features |= Features::SHADER_RESOURCE_MIN_LOD
        };
        if vk_features.sparse_binding == 1 {
            features |= Features::SPARSE_BINDING
        };
        if vk_features.sparse_residency_buffer == 1 {
            features |= Features::SPARSE_RESIDENCY_BUFFER
        };
        if vk_features.sparse_residency_image2_d == 1 {
            features |= Features::SPARSE_RESIDENCY_IMAGE_2D
        };
        if vk_features.sparse_residency_image3_d == 1 {
            features |= Features::SPARSE_RESIDENCY_IMAGE_3D
        };
        if vk_features.sparse_residency2_samples == 1 {
            features |= Features::SPARSE_RESIDENCY_2_SAMPLES
        };
        if vk_features.sparse_residency4_samples == 1 {
            features |= Features::SPARSE_RESIDENCY_4_SAMPLES
        };
        if vk_features.sparse_residency8_samples == 1 {
            features |= Features::SPARSE_RESIDENCY_8_SAMPLES
        };
        if vk_features.sparse_residency16_samples == 1 {
            features |= Features::SPARSE_RESIDENCY_16_SAMPLES
        };
        if vk_features.sparse_residency_aliased == 1 {
            features |= Features::SPARSE_RESIDENCY_ALIASED
        };
        if vk_features.variable_multisample_rate == 1 {
            features |= Features::VARIABLE_MULTISAMPLE_RATE
        };
        if vk_features.inherited_queries == 1 {
            features |= Features::INHERITED_QUERIES
        };

        features
    }
}
impl From<Features> for vk::PhysicalDeviceFeatures {
    fn from(features: Features) -> vk::PhysicalDeviceFeatures {
        let mut vk_features = vk::PhysicalDeviceFeatures::default();

        if features.contains(Features::ROBUST_BUFFER_ACCESS) {
            vk_features.robust_buffer_access = 1
        };
        if features.contains(Features::FULL_DRAW_INDEX_UINT32) {
            vk_features.full_draw_index_uint32 = 1
        };
        if features.contains(Features::IMAGE_CUBE_ARRAY) {
            vk_features.image_cube_array = 1
        };
        if features.contains(Features::INDEPENDENT_BLEND) {
            vk_features.independent_blend = 1
        };
        if features.contains(Features::GEOMETRY_SHADER) {
            vk_features.geometry_shader = 1
        };
        if features.contains(Features::TESSELLATION_SHADER) {
            vk_features.tessellation_shader = 1
        };
        if features.contains(Features::SAMPLE_RATE_SHADING) {
            vk_features.sample_rate_shading = 1
        };
        if features.contains(Features::DUAL_SRC_BLEND) {
            vk_features.dual_src_blend = 1
        };
        if features.contains(Features::LOGIC_OP) {
            vk_features.logic_op = 1
        };
        if features.contains(Features::MULTI_DRAW_INDIRECT) {
            vk_features.multi_draw_indirect = 1
        };
        if features.contains(Features::DRAW_INDIRECT_FIRST_INSTANCE) {
            vk_features.draw_indirect_first_instance = 1
        };
        if features.contains(Features::DEPTH_CLAMP) {
            vk_features.depth_clamp = 1
        };
        if features.contains(Features::DEPTH_BIAS_CLAMP) {
            vk_features.depth_bias_clamp = 1
        };
        if features.contains(Features::FILL_MODE_NON_SOLID) {
            vk_features.fill_mode_non_solid = 1
        };
        if features.contains(Features::DEPTH_BOUNDS) {
            vk_features.depth_bounds = 1
        };
        if features.contains(Features::WIDE_LINES) {
            vk_features.wide_lines = 1
        };
        if features.contains(Features::LARGE_POINTS) {
            vk_features.large_points = 1
        };
        if features.contains(Features::ALPHA_TO_ONE) {
            vk_features.alpha_to_one = 1
        };
        if features.contains(Features::MULTI_VIEWPORT) {
            vk_features.multi_viewport = 1
        };
        if features.contains(Features::SAMPLER_ANISOTROPY) {
            vk_features.sampler_anisotropy = 1
        };
        if features.contains(Features::TEXTURE_COMPRESSION_ETC2) {
            vk_features.texture_compression_etc2 = 1
        };
        if features.contains(Features::TEXTURE_COMPRESSION_ASTC_LDR) {
            vk_features.texture_compression_astc_ldr = 1
        };
        if features.contains(Features::TEXTURE_COMPRESSION_BC) {
            vk_features.texture_compression_bc = 1
        };
        if features.contains(Features::OCCLUSION_QUERY_PRECISE) {
            vk_features.occlusion_query_precise = 1
        };
        if features.contains(Features::PIPELINE_STATISTICS_QUERY) {
            vk_features.pipeline_statistics_query = 1
        };
        if features.contains(Features::VERTEX_PIPELINE_STORES_AND_ATOMICS) {
            vk_features.vertex_pipeline_stores_and_atomics = 1
        };
        if features.contains(Features::FRAGMENT_STORES_AND_ATOMICS) {
            vk_features.fragment_stores_and_atomics = 1
        };
        if features.contains(Features::SHADER_TESSELLATION_AND_GEOMETRY_POINT_SIZE) {
            vk_features.shader_tessellation_and_geometry_point_size = 1
        };
        if features.contains(Features::SHADER_IMAGE_GATHER_EXTENDED) {
            vk_features.shader_image_gather_extended = 1
        };
        if features.contains(Features::SHADER_STORAGE_IMAGE_EXTENDED_FORMATS) {
            vk_features.shader_storage_image_extended_formats = 1
        };
        if features.contains(Features::SHADER_STORAGE_IMAGE_MULTISAMPLE) {
            vk_features.shader_storage_image_multisample = 1
        };
        if features.contains(Features::SHADER_STORAGE_IMAGE_READ_WITHOUT_FORMAT) {
            vk_features.shader_storage_image_read_without_format = 1
        };
        if features.contains(Features::SHADER_STORAGE_IMAGE_WRITE_WITHOUT_FORMAT) {
            vk_features.shader_storage_image_write_without_format = 1
        };
        if features.contains(Features::SHADER_UNIFORM_BUFFER_ARRAY_DYNAMIC_INDEXING) {
            vk_features.shader_uniform_buffer_array_dynamic_indexing = 1
        };
        if features.contains(Features::SHADER_SAMPLED_IMAGE_ARRAY_DYNAMIC_INDEXING) {
            vk_features.shader_sampled_image_array_dynamic_indexing = 1
        };
        if features.contains(Features::SHADER_STORAGE_BUFFER_ARRAY_DYNAMIC_INDEXING) {
            vk_features.shader_storage_buffer_array_dynamic_indexing = 1
        };
        if features.contains(Features::SHADER_STORAGE_IMAGE_ARRAY_DYNAMIC_INDEXING) {
            vk_features.shader_storage_image_array_dynamic_indexing = 1
        };
        if features.contains(Features::SHADER_CLIP_DISTANCE) {
            vk_features.shader_clip_distance = 1
        };
        if features.contains(Features::SHADER_CULL_DISTANCE) {
            vk_features.shader_cull_distance = 1
        };
        if features.contains(Features::SHADER_FLOAT64) {
            vk_features.shader_float64 = 1
        };
        if features.contains(Features::SHADER_INT64) {
            vk_features.shader_int64 = 1
        };
        if features.contains(Features::SHADER_INT16) {
            vk_features.shader_int16 = 1
        };
        if features.contains(Features::SHADER_RESOURCE_RESIDENCY) {
            vk_features.shader_resource_residency = 1
        };
        if features.contains(Features::SHADER_RESOURCE_MIN_LOD) {
            vk_features.shader_resource_min_lod = 1
        };
        if features.contains(Features::SPARSE_BINDING) {
            vk_features.sparse_binding = 1
        };
        if features.contains(Features::SPARSE_RESIDENCY_BUFFER) {
            vk_features.sparse_residency_buffer = 1
        };
        if features.contains(Features::SPARSE_RESIDENCY_IMAGE_2D) {
            vk_features.sparse_residency_image2_d = 1
        };
        if features.contains(Features::SPARSE_RESIDENCY_IMAGE_3D) {
            vk_features.sparse_residency_image3_d = 1
        };
        if features.contains(Features::SPARSE_RESIDENCY_2_SAMPLES) {
            vk_features.sparse_residency2_samples = 1
        };
        if features.contains(Features::SPARSE_RESIDENCY_4_SAMPLES) {
            vk_features.sparse_residency4_samples = 1
        };
        if features.contains(Features::SPARSE_RESIDENCY_8_SAMPLES) {
            vk_features.sparse_residency8_samples = 1
        };
        if features.contains(Features::SPARSE_RESIDENCY_16_SAMPLES) {
            vk_features.sparse_residency16_samples = 1
        };
        if features.contains(Features::SPARSE_RESIDENCY_ALIASED) {
            vk_features.sparse_residency_aliased = 1
        };
        if features.contains(Features::VARIABLE_MULTISAMPLE_RATE) {
            vk_features.variable_multisample_rate = 1
        };
        if features.contains(Features::INHERITED_QUERIES) {
            vk_features.inherited_queries = 1
        };

        vk_features
    }
}
