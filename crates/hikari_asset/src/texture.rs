use graphy::*;

pub struct Texture {
    pub name: String,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub filtering: graphy::FilterMode,
    pub generate_mips: bool,
    pub wrap_x: graphy::WrapMode,
    pub wrap_y: graphy::WrapMode,
    pub format: graphy::Format,
}