use thiserror::Error;
#[derive(Error, Debug)]
pub enum ResourceAllocationError {
    #[error("Failed to create VertexArray")]
    VertexArray,
    #[error("Failed to create buffer")]
    Buffer,
    #[error("Failed to create Texture")]
    Texture,
    #[error("Failed to create Framebuffer: {0}")]
    Framebuffer(&'static str),
}
#[derive(Error, Debug)]
pub enum UnsupportedFeatureError {
    #[error("Feature not supported by device: Anisotropic Filtering")]
    AnisotropicFiltering,
}
