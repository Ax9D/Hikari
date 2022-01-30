use once_cell::sync::OnceCell;

static WHITE_TEXTURE: OnceCell<hikari_asset::Asset<graphy::Texture2D>> = OnceCell::new();
static BLACK_TEXTURE: OnceCell<hikari_asset::Asset<graphy::Texture2D>> = OnceCell::new();
static GREY_TEXTURE: OnceCell<hikari_asset::Asset<graphy::Texture2D>> = OnceCell::new();
static CHECKERBOARD_TEXTURE: OnceCell<hikari_asset::Asset<graphy::Texture2D>> = OnceCell::new();

pub fn init(ctx: &mut crate::Context) -> Result<(), Box<dyn std::error::Error>> {
    if WHITE_TEXTURE.get().is_none() {
        const N: usize = 2;
        let data = [255; N * N * 4];
        let config = graphy::TextureConfig {
            width: N as u32,
            height: N as u32,
            format: graphy::Format::RGBA,
            filtering: graphy::FilterMode::Closest,
            wrap_x: graphy::WrapMode::Repeat,
            wrap_y: graphy::WrapMode::Repeat,
            aniso_level: 0,
        };

        WHITE_TEXTURE.set(hikari_asset::Asset::new(
            "white_texture",
            std::path::Path::new(""),
            graphy::Texture2D::new(ctx.gfx().device(), &data, config)?,
        ));
    }
    if BLACK_TEXTURE.get().is_none() {
        const N: usize = 2;
        let data = [0; N * N * 4];
        let config = graphy::TextureConfig {
            width: N as u32,
            height: N as u32,
            format: graphy::Format::RGBA,
            filtering: graphy::FilterMode::Closest,
            wrap_x: graphy::WrapMode::Repeat,
            wrap_y: graphy::WrapMode::Repeat,
            aniso_level: 0,
        };

        BLACK_TEXTURE.set(hikari_asset::Asset::new(
            "black_texture",
            std::path::Path::new(""),
            graphy::Texture2D::new(ctx.gfx().device(), &data, config)?,
        ));
    }

    if GREY_TEXTURE.get().is_none() {
        const N: usize = 2;
        let data = [127; N * N * 4];
        let config = graphy::TextureConfig {
            width: N as u32,
            height: N as u32,
            format: graphy::Format::RGBA,
            filtering: graphy::FilterMode::Closest,
            wrap_x: graphy::WrapMode::Repeat,
            wrap_y: graphy::WrapMode::Repeat,
            aniso_level: 0,
        };

        GREY_TEXTURE.set(hikari_asset::Asset::new(
            "black_texture",
            std::path::Path::new(""),
            graphy::Texture2D::new(ctx.gfx().device(), &data, config)?,
        ));
    }

    if CHECKERBOARD_TEXTURE.get().is_none() {
        let data = include_bytes!("../checkerboard.data");
        let n = (((data.len() / 4) as f32).sqrt()) as u32;

        let config = graphy::TextureConfig {
            width: n,
            height: n,
            format: graphy::Format::RGBA,
            filtering: graphy::FilterMode::Closest,
            wrap_x: graphy::WrapMode::Repeat,
            wrap_y: graphy::WrapMode::Repeat,
            aniso_level: 8,
        };

        CHECKERBOARD_TEXTURE.set(hikari_asset::Asset::new(
            "checkerboard_texture",
            std::path::Path::new(""),
            graphy::Texture2D::new(ctx.gfx().device(), data, config)?,
        ));
    }

    Ok(())
}
pub fn white() -> &'static hikari_asset::Asset<graphy::Texture2D> {
    &WHITE_TEXTURE.get().unwrap()
}
//[0.5,0.5,0.5]
pub fn grey() -> &'static hikari_asset::Asset<graphy::Texture2D> {
    &GREY_TEXTURE.get().unwrap()
}
pub fn black() -> &'static hikari_asset::Asset<graphy::Texture2D> {
    &BLACK_TEXTURE.get().unwrap()
}
pub fn checkerboard() -> &'static hikari_asset::Asset<graphy::Texture2D> {
    &CHECKERBOARD_TEXTURE.get().unwrap()
}
