use std::borrow::Cow;
use std::ops::DerefMut;
use std::sync::Arc;

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

pub struct Gfx {
    device: Arc<crate::Device>,
    surface: vk::SurfaceKHR,
    surface_loader: Surface,
    swapchain: Arc<Mutex<Swapchain>>, //
    entry: ash::Entry,                //
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
    fn create_instance(entry: &Entry, window: &Window, debug: bool) -> VkResult<ash::Instance> {
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
                .enabled_extension_names(&extension_names);

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
        use vk::DebugUtilsMessageTypeFlagsEXT as mtype;
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(severity::INFO | severity::ERROR | severity::WARNING)
            .message_type(mtype::GENERAL | mtype::PERFORMANCE | mtype::VALIDATION)
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
    pub fn new(window: &Window, config: GfxConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let entry = unsafe { Entry::load() }?;

        log::debug!("Available instance extension properties: ");
        entry
            .enumerate_instance_extension_properties()?
            .iter()
            .for_each(|prop| {
                log::debug!("{:?}", unsafe {
                    CStr::from_ptr(prop.extension_name.as_ptr())
                });
            });

        let instance = Self::create_instance(&entry, window, config.debug)?;

        if config.debug {
            Self::create_debug_messenger(&entry, &instance, Some(vulkan_debug_callback))?;
        }

        let surface = Self::create_surface(&entry, &instance, window)?;
        let surface_loader = Surface::new(&entry, &instance);

        let device =
            crate::Device::create(&entry, instance, &surface, &surface_loader, config.features)?;

        let window_size = window.inner_size();
        let swapchain = crate::Swapchain::create(
            &device,
            window_size.width,
            window_size.height,
            &surface,
            &surface_loader,
            None,
        )?;
        let swapchain = Arc::new(Mutex::new(swapchain));

        Ok(Self {
            entry,
            device,
            surface,
            surface_loader,
            swapchain,
        })
    }
    pub fn device(&self) -> &Arc<crate::Device> {
        //log::debug!("{}", Arc::strong_count(&self.device) + Arc::weak_count(&self.device));
        &self.device
    }
    pub fn swapchain(&self) -> &Arc<Mutex<Swapchain>> {
        &self.swapchain
    }
    pub fn resize(
        &mut self,
        new_width: u32,
        new_height: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            self.device.raw().device_wait_idle()?;
        };
        let mut swapchain = self.swapchain().lock();
        let new_swapchain = Swapchain::create(
            &self.device,
            new_width,
            new_height,
            &self.surface,
            &self.surface_loader,
            Some(swapchain.inner),
        )?;
        let old_swapchain = std::mem::replace(swapchain.deref_mut(), new_swapchain);

        log::debug!("Resized swapchain width: {new_width} height: {new_height}");

        Ok(())
    }
}
impl Drop for Gfx {
    fn drop(&mut self) {
        log::debug!("Dropped gfx");
    }
}
#[derive(Debug, Default)]
pub struct GfxConfig {
    pub debug: bool,
    pub features: crate::Features,
}
