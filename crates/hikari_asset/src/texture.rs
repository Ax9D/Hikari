use graphy::texture::TextureConfig;

pub struct Texture {
    pub(crate) name: String,
    pub(crate) data: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) config: graphy::texture::TextureConfig,
}

impl Texture {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn config(&self) -> TextureConfig {
        self.config.clone()
    }
}
