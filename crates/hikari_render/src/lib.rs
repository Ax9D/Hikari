#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_unsafe)]

#[macro_use]
extern crate const_cstr;

pub mod bindless;
pub mod buffer;
pub mod device;
pub mod error;
pub mod gfx;
pub mod graph;
pub mod image;
pub mod shader;

#[cfg(feature = "imgui-support")]
pub mod imgui_support;

#[cfg(feature = "renderdoc")]
pub mod renderdoc;

pub use device::Device;
pub use device::Features;
pub use error::*;
pub use gfx::Gfx;
pub use gfx::GfxConfig;

pub use crate::image::*;
pub use shader::*;
pub use shaderc;

pub use buffer::*;

pub use graph::*;

mod barrier;
mod delete;
mod descriptor;
mod renderpass;
mod framebuffer;
mod swapchain;
mod util;
mod cache;

use renderpass::PhysicalRenderpass;
use swapchain::Swapchain;

pub use ash::vk;
pub use util::n_workgroups;
pub use util::PerFrame;
pub use vk_sync::AccessType;
pub use vk_sync_fork as vk_sync;

pub struct GfxPlugin {
    pub config: GfxConfig,
}

impl hikari_core::Plugin for GfxPlugin {
    fn build(self, game: &mut hikari_core::Game) {
        let gfx = Gfx::new(&mut game.window(), self.config).expect("Failed to create render context");
        game.add_state(gfx);

        #[cfg(feature = "renderdoc")]
        game.add_plugin(renderdoc::RenderdocPlugin);

        game.add_task(
            hikari_core::FIRST,
            hikari_core::Task::new("Gfx New Frame", |gfx: &mut Gfx| {
                gfx.new_frame().expect("Failed to update gfx");
            }),
        );
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
