use ash::vk;
pub mod compute;
pub mod graphics;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
pub enum ColorFormat {
    R8G8B8A8_UNORM,
    R8G8B8A8_UINT,
    R16G16B16A16_SFLOAT,
    R32G32B32A32_SFLOAT,
}

impl Into<vk::Format> for ColorFormat {
    fn into(self) -> vk::Format {
        match self {
            ColorFormat::R8G8B8A8_UINT => vk::Format::R8G8B8A8_UINT,
            ColorFormat::R32G32B32A32_SFLOAT => vk::Format::R32G32B32A32_SFLOAT,
            ColorFormat::R16G16B16A16_SFLOAT => vk::Format::R16G16B16A16_SFLOAT,
            ColorFormat::R8G8B8A8_UNORM => vk::Format::R8G8B8A8_UNORM,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum DepthStencilFormat {
    D16_UNORM,
    D24_UNORM_S8_UINT,
}
impl Into<vk::Format> for DepthStencilFormat {
    fn into(self) -> vk::Format {
        match self {
            DepthStencilFormat::D16_UNORM => vk::Format::D16_UNORM,
            DepthStencilFormat::D24_UNORM_S8_UINT => vk::Format::D24_UNORM_S8_UINT,
        }
    }
}

impl DepthStencilFormat {
    pub fn is_stencil_format(&self) -> bool {
        match self {
            DepthStencilFormat::D16_UNORM => false,
            DepthStencilFormat::D24_UNORM_S8_UINT => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ImageSize {
    Relative(f32, f32), //Ratio
    Absolute(u32, u32), //Pixel size
}

impl ImageSize {
    pub fn get_size(&self, graph_size: (u32, u32)) -> (u32, u32) {
        match self.clone() {
            ImageSize::Relative(fw, fh) => (
                (fw * graph_size.0 as f32) as u32,
                (fh * graph_size.1 as f32) as u32,
            ),
            ImageSize::Absolute(width, height) => (width, height),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Input {
    Dependency,
    Read(u32),
}
#[derive(Clone)]
pub enum Output {
    Color(graphics::ColorOutput),
    DepthStencil(graphics::DepthStencilOutput),
    StorageBuffer,
}

impl Output {
    pub fn is_graphics(&self) -> bool {
        match self {
            Output::Color(_) => true,
            Output::DepthStencil(_) => true,
            _ => false,
        }
    }
}
