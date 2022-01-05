use graphy::*;

pub struct Texture {
    pub(crate) name: String,
    pub(crate) data: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) filtering: graphy::FilterMode,
    pub(crate) wrap_x: graphy::WrapMode,
    pub(crate) wrap_y: graphy::WrapMode,
    pub(crate) format: graphy::Format,
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
    pub fn filtering(&self) -> FilterMode {
        self.filtering
    }
    pub fn wrap_x(&self) -> WrapMode {
        self.wrap_x
    }
    pub fn wrap_y(&self) -> WrapMode {
        self.wrap_y
    }
    pub fn format(&self) -> Format {
        self.format
    }
}
