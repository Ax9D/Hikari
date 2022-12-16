use ash::vk;
use vk_sync_fork::AccessType;

use crate::texture::SampledImage;

use self::graphics::Renderpass;

use super::{storage::ErasedHandle, GpuHandle};
pub mod compute;
pub mod graphics;

/// Represents the size of an Image
/// Relative means that the image will always be a certain fraction relative to the Graph's size
/// Absolute means that the image will always be of a constant width and height in pixels irrespective of what size the Graph is
///  
/// For e.g. if the output resolution of the graph is (800, 600), and an image is created with `ImageSize::Relative(0.5, 0.4)`,
/// Its physical size will be (0.5 * 800, 0.4 * 600) = (400, 240) pixels; moreover if the graph is resized to say (1920, 1080),
/// the image will be resized to (0.5 * 1920, 0.4 * 1080) = (960, 432)
///
/// If however an image is created with `ImageSize::Absolute(800, 600)`, it will always be of a size (800, 600) no matter the
/// output size of the graph
#[derive(Debug, Clone, Copy)]
pub struct ImageSize {
    pub x: SizeMode,
    pub y: SizeMode,
    pub z: SizeMode
}

impl ImageSize {
    pub fn default_xy() -> Self {
        Self::relative_xy(1.0, 1.0)
    }
    pub fn absolute_xy(x: u32, y: u32) -> Self {
        Self::absolute_xyz(x, y, 1)
    } 
    pub fn absolute_xyz(x: u32, y: u32, z: u32) -> Self {
        ImageSize { x: SizeMode::Absolute(x), y: SizeMode::Absolute(y), z: SizeMode::Absolute(z)}
    }
    pub fn relative_xy(x: f32, y: f32) -> Self {
        ImageSize { x: SizeMode::RelativeX(x), y: SizeMode::RelativeY(y), z: SizeMode::Absolute(1) }
    }
}

impl ImageSize {
    pub fn get_physical_size_2d(&self, graph_size: (u32, u32)) -> (u32, u32) {
        let (x, y, _) = self.get_physical_size_3d(graph_size);
        (x, y)
    }
    pub fn get_physical_size_3d(&self, graph_size: (u32, u32)) -> (u32, u32, u32) {
        (self.x.get_physical_size(graph_size), self.y.get_physical_size(graph_size), self.z.get_physical_size(graph_size))
    }
}
#[derive(Debug, Clone, Copy)]
pub enum SizeMode {
    RelativeX(f32),
    RelativeY(f32),
    Absolute(u32),
}

impl SizeMode {
    fn get_physical_size(&self, graph_size: (u32, u32)) -> u32 {
        match self {
            SizeMode::RelativeX(ratio)  => (ratio * graph_size.0 as f32) as u32,
            SizeMode::RelativeY(ratio) =>(ratio * graph_size.1 as f32) as u32,
            SizeMode::Absolute(value) => *value,
        }
    }
}
/// Defines how the attachment will be used
/// `AttachmentKind::Color(2)` means it is a color attachment which will be addressed at output location 2 in the fragment shader
/// `AttachmentKind::DepthStencil` means it is a depth stencil attachment
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum AttachmentKind {
    Color(u32),
    DepthStencil,
    DepthOnly,
}

#[derive(Debug, Clone, Copy)]
pub struct AttachmentConfig {
    pub kind: AttachmentKind,
    pub access: crate::AccessType,
    pub load_op: vk::AttachmentLoadOp,
    pub store_op: vk::AttachmentStoreOp,
    pub stencil_store_op: vk::AttachmentStoreOp,
    pub stencil_load_op: vk::AttachmentLoadOp,
}
impl AttachmentConfig {
    pub const fn color_default(location: u32) -> Self {
        Self {
            kind: AttachmentKind::Color(location),
            access: crate::AccessType::ColorAttachmentWrite,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,

            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        }
    }
    pub const fn depth_only_default() -> Self {
        Self {
            kind: AttachmentKind::DepthOnly,
            access: crate::AccessType::DepthStencilAttachmentWrite,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,

            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        }
    }
    pub const fn depth_stencil_default() -> Self {
        Self {
            kind: AttachmentKind::DepthStencil,
            access: crate::AccessType::DepthStencilAttachmentWrite,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,

            stencil_load_op: vk::AttachmentLoadOp::CLEAR,
            stencil_store_op: vk::AttachmentStoreOp::STORE,
        }
    }
}

// #[derive(Debug, Clone)]
// pub struct BufferInfo {
//     raw: vk::Buffer,
//     offset: u64,
//     len: u64
// }

#[derive(Debug, Clone)]
pub enum Input {
    ReadImage(GpuHandle<SampledImage>, AccessType),
    ReadStorageBuffer(ErasedHandle, AccessType),
}

impl Input {
    pub fn erased_handle(&self) -> ErasedHandle {
        match self {
            Input::ReadImage(handle, _) => handle.clone().into(),
            Input::ReadStorageBuffer(handle, _) => handle.clone(),
        }
    }
}
#[derive(Debug, Clone)]
pub enum Output {
    WriteImage(GpuHandle<SampledImage>, AccessType),
    DrawImage(GpuHandle<SampledImage>, AttachmentConfig),
    WriteStorageBuffer(ErasedHandle, AccessType),
}

impl Output {
    pub fn erased_handle(&self) -> ErasedHandle {
        match self {
            Output::WriteImage(handle, _) | Output::DrawImage(handle, _) => handle.clone().into(),
            _ => todo!(),
        }
    }
}

pub enum AnyPass<T: crate::Args> {
    Render(Renderpass<T>),
    Compute(u32),
}

impl<T: crate::Args> AnyPass<T> {
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
