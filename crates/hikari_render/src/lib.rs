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
pub use imgui;
pub use util::PerFrame;
pub use vk_sync_fork::AccessType;
