use ash::vk;
use vk_sync_fork::AccessType;

use crate::texture::SampledImage;

use self::graphics::Renderpass;

use super::{storage::ErasedHandle, Handle};
pub mod compute;
pub mod graphics;

#[derive(Debug, Clone, Copy)]
pub enum ImageSize {
    Relative(f32, f32), //Ratio
    Absolute(u32, u32), //Pixel size
}

impl Default for ImageSize {
    fn default() -> Self {
        ImageSize::Relative(1.0, 1.0)
    }
}

impl ImageSize {
    pub fn get_physical_size(&self, graph_size: (u32, u32)) -> (u32, u32) {
        match self.clone() {
            ImageSize::Relative(fw, fh) => (
                (fw * graph_size.0 as f32) as u32,
                (fh * graph_size.1 as f32) as u32,
            ),
            ImageSize::Absolute(width, height) => (width, height),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum AttachmentKind {
    Color(u32),
    DepthStencil,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AttachmentConfig {
    pub kind: AttachmentKind,
    pub load_op: vk::AttachmentLoadOp,
    pub store_op: vk::AttachmentStoreOp,
    pub stencil_store_op: vk::AttachmentStoreOp,
    pub stencil_load_op: vk::AttachmentLoadOp,
}
impl AttachmentConfig {
    pub const fn color_default(location: u32) -> Self {
        Self {
            kind: AttachmentKind::Color(location),
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,

            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        }
    }
    pub const fn depth_only_default() -> Self {
        Self {
            kind: AttachmentKind::DepthStencil,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,

            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        }
    }
    pub const fn depth_stencil_default() -> Self {
        Self {
            kind: AttachmentKind::DepthStencil,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,

            stencil_load_op: vk::AttachmentLoadOp::CLEAR,
            stencil_store_op: vk::AttachmentStoreOp::STORE,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Input {
    ReadImage(Handle<SampledImage>, AccessType),
    SampleImage(Handle<SampledImage>, AccessType, u32),
}

impl Input {
    pub fn erased_handle(&self) -> ErasedHandle {
        match self {
            Input::ReadImage(handle, _) | Input::SampleImage(handle, _, _) => handle.clone().into(),
        }
    }
}
#[derive(Debug, Clone)]
pub enum Output {
    WriteImage(Handle<SampledImage>, AccessType),
    DrawImage(Handle<SampledImage>, AttachmentConfig),
    StorageBuffer,
}

impl Output {
    pub fn erased_handle(&self) -> ErasedHandle {
        match self {
            Output::WriteImage(handle, _) | Output::DrawImage(handle, _) => handle.clone().into(),
            _ => todo!(),
        }
    }
}

pub enum AnyPass<S, P, R> {
    Render(Renderpass<S, P, R>),
    Compute(u32),
}

impl<S, P, R> AnyPass<S, P, R> {
    pub fn name(&self) -> &str {
        match self {
            AnyPass::Render(pass) => pass.name(),
            AnyPass::Compute(_) => todo!(),
        }
    }
    pub fn id(&self) -> u64 {
        match self {
            AnyPass::Render(pass) => pass.id(),
            AnyPass::Compute(_) => todo!(),
        }
    }
    pub fn inputs(&self) -> &[Input] {
        match self {
            AnyPass::Render(pass) => pass.inputs(),
            AnyPass::Compute(_) => todo!(),
        }
    }
    pub fn outputs(&self) -> &[Output] {
        match self {
            AnyPass::Render(pass) => pass.outputs(),
            AnyPass::Compute(_) => todo!(),
        }
    }
}
