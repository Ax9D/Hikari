use ::image::EncodableLayout;

use crate::config::*;
use std::sync::Arc;

use hikari_asset::{Asset, LoadContext, Loader};
use hikari_render::*;

#[derive(type_uuid::TypeUuid)]
#[uuid = "70ff2d10-3fc1-4851-9fb7-521d8cd49ad5"]
pub struct Texture2D {
    image: SampledImage,
    config: TextureConfig,
}
impl Texture2D {
    pub fn new<T: Copy>(
        device: &Arc<hikari_render::Device>,
        data: &[T],
        width: u32,
        height: u32,
        config: TextureConfig,
    ) -> Result<Texture2D, anyhow::Error> {
        Ok(Self {
            image: SampledImage::with_data(
                device,
                data,
                width,
                height,
                1,
                config.into_image_config_2d(width, height)?,
            )?,
            config,
        })
    }
    pub fn with_dimensions(
        device: &Arc<hikari_render::Device>,
        width: u32,
        height: u32,
        config: TextureConfig,
    ) -> Result<Texture2D, anyhow::Error> {
        Ok(Self {
            image: SampledImage::with_dimensions(
                device,
                width,
                height,
                1,
                1,
                config.into_image_config_2d(width, height)?,
            )?,
            config,
        })
    }
    pub fn from_parts(image: SampledImage, config: TextureConfig) -> Self {
        Self { image, config }
    }
    pub fn raw(&self) -> &SampledImage {
        &self.image
    }
    pub fn width(&self) -> u32 {
        self.image.width()
    }
    pub fn height(&self) -> u32 {
        self.image.height()
    }
    pub fn config(&self) -> &TextureConfig {
        &self.config
    }
}

pub const SUPPORTED_TEXTURE_EXTENSIONS: [&'static str; 7] =
    ["png", "jpg", "jpeg", "dds", "bmp", "gif", "tga"];
pub struct TextureLoader {
    pub device: Arc<Device>,
}

impl Asset for Texture2D {
    type Settings = TextureConfig;
}
impl Loader for TextureLoader {
    fn load(&self, context: &mut LoadContext) -> anyhow::Result<()> {
        //let mut raw_data = vec![];
        //context.reader().read_to_end(&mut raw_data)?;

        let format = ::image::ImageFormat::from_path(context.path())?;

        let image = ::image::load(context.reader(), format)?;
        //let image = image::load_from_memory(&buf_reader)?;
        let width = image.width();
        let height = image.height();

        let config = *context.settings::<Texture2D>();

        let image = image.to_rgba8();
        let data = image.as_bytes();

        let texture = Texture2D::new(&self.device, data, width, height, config)?;
        context.set_asset(texture);

        Ok(())
    }

    fn extensions(&self) -> &[&str] {
        &SUPPORTED_TEXTURE_EXTENSIONS
    }
}

#[test]
fn parallel_load() -> Result<(), Box<dyn std::error::Error>> {
    use crate::*;
    use rayon::prelude::*;

    let gfx = Gfx::headless(GfxConfig {
        debug: true,
        features: Features::default(),
        vsync: false,
    })?;

    let path = "../../engine_assets/models/sponza/14930275953430797156.png";
    let (image, width, height) = image::open_rgba8(path).unwrap();

    let _textures: Vec<_> = (0..1000)
        .into_par_iter()
        .map(|_| {
            Texture2D::new(
                gfx.device(),
                &image,
                width,
                height,
                TextureConfig::default(),
            )
            .unwrap()
        })
        .collect();
    Ok(())
}
