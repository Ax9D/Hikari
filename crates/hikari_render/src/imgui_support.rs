use once_cell::sync::OnceCell;
use std::collections::HashMap;

use std::time::Instant;
use std::{ptr::NonNull, sync::Arc};

use ash::{prelude::VkResult, vk};

use hikari_imgui as imgui;
use hikari_imgui::imgui_winit_support::{HiDpiMode, WinitPlatform};
use imgui::imgui_rs_vulkan_renderer::{self, Options};
use parking_lot::Mutex;
use winit::{event::Event, window::Window};

use crate::descriptor::{DescriptorSetAllocator, DescriptorSetLayout, DescriptorSetState};
use crate::SampledImage;

unsafe impl Send for SharedDrawData {}
unsafe impl Sync for SharedDrawData {}
/// Provides shared access to imgui::DrawData
/// Useful when update and rendering need to be performed separately
/// Clone SharedDrawData from the Backend
/// Call `new_frame_shared(...)` on the backend to update the draw data
/// Pass your SharedDrawData to the renderer using `render_from_shared(...)`
pub struct SharedDrawData {
    inner: Arc<Mutex<Option<NonNull<imgui::DrawData>>>>,
}
impl SharedDrawData {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }
    pub(crate) fn set(&self, raw_draw_data: *mut imgui::DrawData) {
        let non_null = NonNull::new(raw_draw_data).expect("imgui::DrawData is null");
        self.inner.lock().replace(non_null);
    }
    pub(crate) fn get(&self) -> Option<&imgui::DrawData> {
        unsafe { self.inner.lock().map(|raw| raw.as_ref()) }
    }
}

impl Clone for SharedDrawData {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

unsafe impl Send for Backend {}
unsafe impl Sync for Backend {}
pub struct Backend {
    imgui: imgui::Context,
    platform: WinitPlatform,
    draw_data: SharedDrawData,
    last_frame: Instant
}

impl Backend {
    pub fn new(
        window: &mut Window,
        mut imgui: imgui::Context,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut platform = WinitPlatform::init(&mut imgui);

        let hidpi_factor = window.scale_factor();
        //imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);

        Ok(Self {
            imgui,
            platform,
            draw_data: SharedDrawData::new(),
            last_frame: Instant::now(),
        })
    }
    #[inline]
    pub fn hidpi_factor(&self) -> f64 {
        self.platform.hidpi_factor()
    }
    pub fn shared_draw_data(&self) -> &SharedDrawData {
        &self.draw_data
    }
    #[inline]
    pub fn context(&mut self) -> &mut imgui::Context {
        &mut self.imgui
    }
    #[inline]
    pub fn handle_event<T>(&mut self, window: &Window, event: &Event<T>) {
        match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                let delta = now - self.last_frame;
                self.context().io_mut().update_delta_time(delta);

                self.last_frame = now;
            },
            _=> {}
        }
        self.platform
            .handle_event(self.imgui.io_mut(), window, event);
    }
    pub fn new_frame<'a>(
        &'a mut self,
        window: &Window,
        mut run_fn: impl FnMut(&imgui::Ui),
    ) -> &'a imgui::DrawData {
        self.platform
            .prepare_frame(self.imgui.io_mut(), window)
            .expect("Failed to prepare window for imgui");

        let ui = self.imgui.new_frame();

        (run_fn)(ui);

        self.platform.prepare_render(ui, window);

        self.imgui.render()
    }
    #[allow(clippy::needless_lifetimes)]
    pub fn new_frame_shared<'a>(&'a mut self, window: &Window, mut run_fn: impl FnMut(&imgui::Ui)) {
        self.platform
            .prepare_frame(self.imgui.io_mut(), window)
            .expect("Failed to prepare window for imgui");

        let ui = self.imgui.new_frame();

        (run_fn)(ui);

        self.platform.prepare_render(ui, window);

        unsafe {
            imgui::sys::igRender();
            self.draw_data
                .set(imgui::sys::igGetDrawData() as *mut imgui::DrawData);
        }
    }
}
pub struct Renderer {
    device: Arc<crate::Device>,
    renderer: imgui_rs_vulkan_renderer::Renderer,
    compatible_renderpass: vk::RenderPass,
    textures: TextureMap,
}

impl Renderer {
    pub fn imgui_compatible_renderpass(
        device: &Arc<crate::Device>,
        color_format: vk::Format,
        depth_format: vk::Format,
        present: bool,
    ) -> VkResult<vk::RenderPass> {
        log::debug!("Creating imgui render pass");
        let attachment_descs = [
            vk::AttachmentDescription::builder()
                .format(color_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(if present {
                    vk::ImageLayout::PRESENT_SRC_KHR
                } else {
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
                })
                .build(),
            *vk::AttachmentDescription::builder()
                .format(depth_format)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .samples(vk::SampleCountFlags::TYPE_1)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL),
        ];

        let color_attachment_refs = [vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];

        let depth_stencil_attachment_ref = *vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let subpass_descs = if present {
            [vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachment_refs)
                .depth_stencil_attachment(&depth_stencil_attachment_ref)
                .build()]
        } else {
            [vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachment_refs)
                //.depth_stencil_attachment(&depth_stencil_attachment_ref)
                .build()]
        };

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachment_descs)
            .subpasses(&subpass_descs);

        Ok(unsafe { device.raw().create_render_pass(&render_pass_info, None)? })
    }
    pub fn new(
        device: &Arc<crate::Device>,
        backend: &mut Backend,
        color_format: vk::Format,
        depth_format: vk::Format,
        present: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Command pool to create buffers to upload textures
        let transfer_command_pool = {
            let command_pool_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(device.unified_queue_ix)
                .flags(vk::CommandPoolCreateFlags::empty());
            unsafe { device.raw().create_command_pool(&command_pool_info, None)? }
        };

        let compatible_renderpass =
            Self::imgui_compatible_renderpass(device, color_format, depth_format, present)?;
        let renderer = imgui_rs_vulkan_renderer::Renderer::with_gpu_allocator(
            device.allocator().clone(),
            device.raw().clone(),
            *device.unified_queue(),
            transfer_command_pool,
            compatible_renderpass,
            &mut backend.imgui,
            Some(Options {
                in_flight_frames: 2,
                ..Default::default()
            }),
        )?;
        unsafe {
            device
                .raw()
                .destroy_command_pool(transfer_command_pool, None);
        };

        Ok(Self {
            device: device.clone(),
            renderer,
            compatible_renderpass,
            textures: TextureMap::new(&device)?,
        })
    }
    /// Assumes that a compatible renderpass has been started
    /// It is the caller's responsibility to set the viewport and scissor of the renderpass
    /// The renderpass must also be ended by the caller
    pub fn render(
        &mut self,
        cmd: vk::CommandBuffer,
        draw_data: &imgui::DrawData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.renderer.cmd_draw(cmd, draw_data)?;
        Ok(())
    }
    /// Same as render but takes SharedDrawData
    pub fn render_from_shared(
        &mut self,
        cmd: vk::CommandBuffer,
        draw_data: &SharedDrawData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let draw_data = draw_data
            .get()
            .expect("Draw Data not provided. Was a new_frame_shared called?");
        self.renderer.cmd_draw(cmd, draw_data)?;
        Ok(())
    }

    pub fn get_texture_id(&mut self, sampled_image: &SampledImage) -> imgui::TextureId {
        if let Some(texture_id) = self.textures.get_texture_id(sampled_image) {
            return texture_id;
        }

        let set = self.textures.get_descriptor_set(sampled_image);
        let new_id = self.renderer.textures().insert(set);

        self.textures.register_texture_id(sampled_image, new_id);

        new_id
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .raw()
                .destroy_render_pass(self.compatible_renderpass, None);
        }
    }
}

pub trait TextureExt {
    fn initialize_texture_support(renderer: Arc<Mutex<Renderer>>);
    fn get_texture_id(&self, image: &SampledImage) -> imgui::TextureId;
}
static IMGUI_RENDERER: OnceCell<Arc<Mutex<Renderer>>> = OnceCell::new();

impl TextureExt for imgui::Ui {
    fn initialize_texture_support(renderer: Arc<Mutex<Renderer>>) {
        let _ = IMGUI_RENDERER.set(renderer);
    }
    fn get_texture_id(&self, image: &SampledImage) -> imgui::TextureId {
        hikari_dev::profile_function!();
        IMGUI_RENDERER
            .get()
            .expect("Renderer has not been initialized")
            .lock()
            .get_texture_id(image)
    }
}

struct TextureMap {
    device: Arc<crate::Device>,
    set_allocator: DescriptorSetAllocator,
    set_state: DescriptorSetState,
    image_to_set: HashMap<vk::ImageView, vk::DescriptorSet, crate::util::BuildHasher>,
    image_to_id: HashMap<vk::ImageView, imgui::TextureId, crate::util::BuildHasher>,
}

impl TextureMap {
    pub fn new(device: &Arc<crate::Device>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut set_layout = DescriptorSetLayout::builder();
        set_layout.with_binding(
            0,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            1,
            vk::ShaderStageFlags::FRAGMENT,
        );

        let set_layout = device.set_layout_cache().get_layout(&set_layout)?;

        let set_allocator = DescriptorSetAllocator::new(device, set_layout)?;

        let set_state = DescriptorSetState::new(0, set_layout);
        Ok(Self {
            device: device.clone(),
            set_allocator,
            set_state,
            image_to_set: Default::default(),
            image_to_id: Default::default(),
        })
    }
    pub fn register_texture_id(&mut self, image: &SampledImage, id: imgui::TextureId) {
        self.image_to_id.insert(image.image_view(1).unwrap(), id);
    }
    pub fn get_texture_id(&self, image: &SampledImage) -> Option<imgui::TextureId> {
        self.image_to_id.get(&image.image_view(1).unwrap()).cloned()
    }
    pub fn get_descriptor_set(&mut self, image: &SampledImage) -> vk::DescriptorSet {
        let set_allocator = &mut self.set_allocator;
        let set_state = &mut self.set_state;
        let image_view = image.image_view(1).unwrap();
        let sampler = image.sampler();
        let set = self.image_to_set.entry(image_view).or_insert_with(|| {
            set_state.set_image(0, 0, image_view, sampler);
            set_allocator.get(&set_state)
        });

        *set
    }
}
