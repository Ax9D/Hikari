use std::borrow::Cow;
use std::mem::ManuallyDrop;
use std::{collections::HashSet, sync::Arc};

#[macro_export]
macro_rules! rawToStr {
    ($raw: expr) => {
        unsafe {
            std::ffi::CStr::from_ptr($raw as *const i8)
                .to_str()
                .unwrap()
        }
    };
}

use std::ffi::{CStr, CString};

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;
use ash::prelude::VkResult;
use ash::{vk, Entry};
use parking_lot::Mutex;
use winit::window::Window;

use crate::swapchain::Swapchain;

pub struct DebugSettings {
    panic_on_validation_error: bool,
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!(
                "{bold}[Vulkan]{reset} {:?}\n [{}({})]:{}{reset}",
                message_type,
                message_id_name,
                &message_id_number.to_string(),
                message,
                bold = crossterm::style::Attribute::Bold,
                reset = crossterm::style::Attribute::Reset,
            );
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            log::debug!(
                "{bold}[Vulkan]{reset} {:?}\n [{}({})]:{}{reset}",
                message_type,
                message_id_name,
                &message_id_number.to_string(),
                message,
                bold = crossterm::style::Attribute::Bold,
                reset = crossterm::style::Attribute::Reset,
            );
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!(
                "{bold}[Vulkan]{reset} {:?}\n [{}({})]:{}{reset}",
                message_type,
                message_id_name,
                &message_id_number.to_string(),
                message,
                bold = crossterm::style::Attribute::Bold,
                reset = crossterm::style::Attribute::Reset,
            );
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!(
                "{bold}[Vulkan]{reset} {:?}\n [{}({})]:{}{reset}",
                message_type,
                message_id_name,
                &message_id_number.to_string(),
                message,
                bold = crossterm::style::Attribute::Bold,
                reset = crossterm::style::Attribute::Reset,
            );
        }
        _ => {
            // log::info!("{bold}[Vulkan]{reset} {:?}\n [{}({})]:{}{reset}",
            // message_type,
            // message_id_name,
            // &message_id_number.to_string(),
            // message,
            // bold = crossterm::style::Attribute::Bold,
            // reset = crossterm::style::Attribute::Reset,
            // );
        }
    }

    vk::FALSE
}

fn setup_logging() {
    let colors_line = fern::colors::ColoredLevelConfig::new()
        .error(fern::colors::Color::Red)
        .warn(fern::colors::Color::Yellow)
        .info(fern::colors::Color::Green)
        .debug(fern::colors::Color::BrightBlue)
        .trace(fern::colors::Color::BrightBlack);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{bold}{white}[{}]{reset} {}{s_reset} {}\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors_line.color(record.level()),
                message,
                bold = crossterm::style::Attribute::Bold,
                white = crossterm::style::SetForegroundColor(crossterm::style::Color::White),
                reset = crossterm::style::ResetColor,
                s_reset = crossterm::style::Attribute::Reset
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log").unwrap())
        .apply()
        .unwrap();
}
pub(crate) struct FrameData {
    pub render_semaphore: vk::Semaphore,
    pub present_semaphore: vk::Semaphore,
    pub render_finished_fence: vk::Fence,

    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
}
impl FrameData {
    pub fn new(device: &Arc<crate::Device>) -> VkResult<Self> {
        unsafe {
            let create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(device.physical_device().graphics_queue_index())
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

            let command_pool = device.raw().create_command_pool(&create_info, None)?;

            let create_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .command_buffer_count(1)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffer = device.raw().allocate_command_buffers(&create_info)?[0];

            let create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

            let render_finished_fence = device.raw().create_fence(&create_info, None)?;

            let create_info =
                vk::SemaphoreCreateInfo::builder().flags(vk::SemaphoreCreateFlags::empty());

            let render_semaphore = device.raw().create_semaphore(&create_info, None)?;
            let present_semaphore = device.raw().create_semaphore(&create_info, None)?;

            Ok(Self {
                render_semaphore,
                present_semaphore,
                render_finished_fence,
                command_pool,
                command_buffer,
            })
        }
    }
    pub unsafe fn delete(&self, device: &Arc<crate::Device>) {
        unsafe {
            device.raw().destroy_command_pool(self.command_pool, None);
            device.raw().destroy_fence(self.render_finished_fence, None);
            device.raw().destroy_semaphore(self.render_semaphore, None);
            device.raw().destroy_semaphore(self.present_semaphore, None);
        }

        log::debug!("Deleted Framedata");
    }
}
pub(crate) struct FrameState {
    frame_number: usize,
    frames: [FrameData; 2],
}

impl FrameState {
    pub fn new(device: &Arc<crate::Device>) -> VkResult<Self> {
        Ok(Self {
            frame_number: 1,
            frames: [FrameData::new(device)?, FrameData::new(device)?],
        })
    }
    pub fn current_frame(&self) -> &FrameData {
        &self.frames[(self.frame_number % 2)]
    }
    pub fn last_frame(&self) -> &FrameData {
        &self.frames[(self.frame_number.wrapping_sub(1) % 2)]
    }
    pub fn current_frame_number(&self) -> usize {
        self.frame_number
    }
    pub fn update(&mut self) {
        self.frame_number = self.frame_number.wrapping_add(1);
    }

    pub unsafe fn delete(&self, device: &Arc<crate::Device>) {
        for frame in &self.frames {
            frame.delete(device);
        }
    }
}

pub struct Gfx {
    frame_state: FrameState,
    device: Arc<crate::Device>,
    swapchain: Arc<Mutex<Swapchain>>, //
    entry: ash::Entry,                      //
}
impl Gfx {
    fn get_extensions(window: &Window, debug: bool) -> Vec<*const i8> {
        let mut base_extensions = ash_window::enumerate_required_extensions(window).unwrap();

        if debug {
            base_extensions.push(DebugUtils::name());
        }

        log::debug!("Instance extensions: \n {:?}", base_extensions);

        base_extensions.iter().map(|x| x.as_ptr()).collect()
    }
    fn create_instance(
        entry: &Entry,
        window: &Window,
        debug: bool,
    ) -> Result<ash::Instance, ash::InstanceError> {
        unsafe {
            let app_name = CString::new("Hikari").unwrap();

            let app_info = vk::ApplicationInfo::builder()
                .api_version(vk::make_api_version(0, 1, 2, 0))
                .application_name(&app_name)
                .engine_name(&app_name)
                .engine_version(vk::make_api_version(0, 69, 420, 0));

            let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
            let layer_names: Vec<_> = layer_names.iter().map(|s| s.as_ptr()).collect();

            let extension_names = Self::get_extensions(window, debug);

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .enabled_extension_names(&&extension_names);

            #[cfg(debug_assertions)]
            let create_info = create_info.enabled_layer_names(&layer_names);

            entry.create_instance(&create_info, None)
        }
    }
    fn create_debug_messenger(
        entry: &ash::Entry,
        instance: &ash::Instance,
        callback: vk::PFN_vkDebugUtilsMessengerCallbackEXT,
    ) -> VkResult<vk::DebugUtilsMessengerEXT> {
        use vk::DebugUtilsMessageSeverityFlagsEXT as severity;
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(severity::INFO | severity::ERROR | severity::WARNING)
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .pfn_user_callback(callback);
        let debug_utils_loader = DebugUtils::new(entry, instance);

        unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None) }
    }
    fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &Window,
    ) -> Result<vk::SurfaceKHR, ash::vk::Result> {
        unsafe { ash_window::create_surface(entry, instance, window, None) }
    }
    pub fn new(window: &Window, debug: bool) -> Result<Self, Box<dyn std::error::Error>> {
        setup_logging();

        let entry = unsafe { Entry::new() }?;

        log::debug!("Available instance extension properties: ");
        entry
            .enumerate_instance_extension_properties()?
            .iter()
            .for_each(|prop| {
                log::debug!("{:?}", unsafe {
                    CStr::from_ptr(prop.extension_name.as_ptr())
                });
            });

        let instance = Self::create_instance(&entry, window, debug)?;

        if debug {
            Self::create_debug_messenger(&entry, &instance, Some(vulkan_debug_callback))?;
        }

        let surface = Self::create_surface(&entry, &instance, &window)?;
        let surface_loader = Surface::new(&entry, &instance);

        let device = crate::Device::create(&entry, instance, &surface, &surface_loader)?;

        let swapchain = crate::Swapchain::create(&device, window, &surface, surface_loader)?;
        let swapchain = Arc::new(Mutex::new(swapchain));

        let frame_state = FrameState::new(&device)?;
        Ok(Self {
            entry,
            device,
            swapchain,
            frame_state,
        })
    }
    pub fn device(&self) -> &Arc<crate::Device> {
        //log::debug!("{}", Arc::strong_count(&self.device) + Arc::weak_count(&self.device));
        &self.device
    }
    pub(crate) fn swapchain(&self) -> &Arc<Mutex<Swapchain>> {
        &self.swapchain
    }
    pub(crate) fn frame_state(&self) -> &FrameState {
        &self.frame_state
    }
    pub(crate) fn frame_state_mut(&mut self) -> &mut FrameState {
        &mut self.frame_state
    }
}
impl Drop for Gfx {
    fn drop(&mut self) {
        log::debug!("Dropping FrameState");
        unsafe {
            self.frame_state().delete(self.device());
        }

        log::debug!("Dropped gfx");
    }
}
