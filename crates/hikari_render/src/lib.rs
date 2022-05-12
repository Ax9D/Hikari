#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_unsafe)]

#[macro_use]
extern crate const_cstr;

pub mod buffer;
pub mod device;
pub mod error;
pub mod gfx;
pub mod graph;
pub mod shader;
pub mod texture;

#[cfg(feature = "imgui-support")]
pub mod imgui_support;

pub use device::Device;
pub use device::Features;
pub use error::*;
pub use gfx::Gfx;
pub use gfx::GfxConfig;

pub use shader::*;
pub use texture::sampled_image::*;
//pub use texture::Texture2D;

pub use buffer::*;

pub use graph::*;

mod swapchain;
use swapchain::Swapchain;

mod barrier;
mod descriptor;
mod renderpass;
mod util;

use renderpass::PhysicalRenderpass;

pub use ash::vk;
#[cfg(feature = "imgui-support")]
pub use imgui;
pub use util::PerFrame;
pub use vk_sync_fork::AccessType;

pub struct GfxPlugin {
    pub config: GfxConfig,
}

impl hikari_core::Plugin for GfxPlugin {
    fn build(self, game: &mut hikari_core::Game) {
        let gfx = Gfx::new(game.window(), self.config).expect("Failed to create render context");
        game.add_state(gfx);

        game.add_platform_event_hook(|state, window, event, _control| match event {
            winit::event::Event::WindowEvent { window_id, event } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    if !(size.width == 0 || size.height == 0) {
                        state
                            .get_mut::<Gfx>()
                            .unwrap()
                            .resize(size.width, size.height)
                            .expect("Failed to resize swapchain");
                    }
                }
                _ => {}
            },
            _ => {}
        });
    }
}
