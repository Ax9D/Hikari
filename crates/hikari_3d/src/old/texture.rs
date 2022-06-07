use crate::texture::*;

pub struct Texture {
    pub name: String,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub filtering: FilterMode,
    pub generate_mips: bool,
    pub wrap_x: WrapMode,
    pub wrap_y: WrapMode,
    pub format: Format,
}
