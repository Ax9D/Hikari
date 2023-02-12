use image::EncodableLayout;

use std::{sync::Arc};
use crate::config::*;

use hikari_asset::{Asset, LoadContext, Loader};
use hikari_render::*;

fn into_vk_config(config: &TextureConfig, width: u32, height: u32) -> ImageConfig {
    let format = match config.format {
        //Format::RGB8 => vk::Format::R8G8B8_SNORM,
        Format::RGBA8 => vk::Format::R8G8B8A8_UNORM,
        //Format::SRGB => vk::Format::R8G8B8_SRGB,
        Format::SRGBA => vk::Format::R8G8B8A8_SRGB,
        Format::RGBAFloat16 => vk::Format::R16G16B16A16_SFLOAT,
        Format::RGBAFloat32 => vk::Format::R32G32B32A32_SFLOAT,
    };
    let filtering = match config.filtering {
        FilterMode::Closest => vk::Filter::NEAREST,
        FilterMode::Linear => vk::Filter::LINEAR,
    };
    let wrap_x = match config.wrap_x {
        WrapMode::Clamp => vk::SamplerAddressMode::CLAMP_TO_EDGE,
        WrapMode::Repeat => vk::SamplerAddressMode::REPEAT,
    };
    let wrap_y = match config.wrap_y {
        WrapMode::Clamp => vk::SamplerAddressMode::CLAMP_TO_EDGE,
        WrapMode::Repeat => vk::SamplerAddressMode::REPEAT,
    };
    let wrap_z = vk::SamplerAddressMode::REPEAT;

    let mip_filtering = match config.filtering {
        FilterMode::Closest => vk::SamplerMipmapMode::NEAREST,
        FilterMode::Linear => vk::SamplerMipmapMode::LINEAR,
    };

    ImageConfig {
        format,
        filtering,
        wrap_x,
        wrap_y,
        wrap_z,
        sampler_reduction_mode: None,
        aniso_level: config.aniso_level,
        mip_levels: if config.generate_mips {
            TextureConfig::get_mip_count(width, height)
        } else {
            1
        },
        mip_filtering,
        usage: vk::ImageUsageFlags::SAMPLED,
        flags: vk::ImageCreateFlags::empty(),
        image_type: vk::ImageType::TYPE_2D,
        image_view_type: vk::ImageViewType::TYPE_2D,
        initial_layout: vk::ImageLayout::UNDEFINED,
        host_readable: false,
    }
}
pub struct Texture2D {
    image: SampledImage,
    config: TextureConfig,
}
impl Texture2D {
    pub fn new(
        device: &Arc<hikari_render::Device>,
        data: &[u8],
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
                into_vk_config(&config, width, height),
            )?,
            config,
        })
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

        let format = image::ImageFormat::from_path(context.path())?;

        let image = image::load(context.reader(), format)?;
        //let image = image::load_from_memory(&buf_reader)?;
        let width = image.width();
        let height = image.height();

        let config = *context.settings::<Texture2D>();

        let image = image.to_rgba8();
        let data = image.as_bytes();

        let texture = Texture2D::new(
            &self.device,
            data,
            width,
            height,
            config,
        )?;
        context.set_asset(texture);

        Ok(())
    }

    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "dds", "bmp", "gif", "tga"]
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
