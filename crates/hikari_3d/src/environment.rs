use std::sync::Arc;

use hikari_asset::Asset;
use hikari_asset::Handle;
use hikari_asset::Loader;
use hikari_render::n_workgroups;
use hikari_render::vk;
use hikari_render::vk_sync::ImageLayout;
use hikari_render::AccessType;
use hikari_render::ComputePass;
use hikari_render::Device;
use hikari_render::Gfx;
use hikari_render::Graph;
use hikari_render::GraphBuilder;
use hikari_render::ImageViewDesc;
use hikari_render::SampledImage;
use parking_lot::Mutex;

use crate::config::*;
use crate::ShaderLibrary;
use crate::Texture2D;

type GraphParams = (SampledImage, SampledImage, SampledImage, SampledImage);

pub struct EnvironmentTextureConfig {}

#[derive(type_uuid::TypeUuid)]
#[uuid = "1928ab7d-2dfc-438e-ae23-bf1af8e9866e"]
pub struct EnvironmentTexture {
    skybox: SampledImage,
    diffuse_irradiance: SampledImage,
    specular_prefiltered: SampledImage,
}
impl EnvironmentTexture {
    pub fn skybox(&self) -> &SampledImage {
        &self.skybox
    }
    pub fn diffuse_irradiance(&self) -> &SampledImage {
        &self.diffuse_irradiance
    }
    pub fn specular_prefiltered(&self) -> &SampledImage {
        &self.specular_prefiltered
    }
}
impl Asset for EnvironmentTexture {
    type Settings = ();
}

pub const SUPPORTED_ENV_TEXTURE_EXTENSIONS: [&'static str; 1] = ["hdr"];
pub struct EnvironmentTextureLoader {
    loader: HDRLoader,
}
impl EnvironmentTextureLoader {
    pub fn new(gfx: &mut Gfx, shader_lib: &mut ShaderLibrary) -> anyhow::Result<Self> {
        Ok(Self {
            loader: HDRLoader::new(gfx, shader_lib)?,
        })
    }
}
impl Loader for EnvironmentTextureLoader {
    fn extensions(&self) -> &[&str] {
        &SUPPORTED_ENV_TEXTURE_EXTENSIONS
    }

    fn load(&self, ctx: &mut hikari_asset::LoadContext) -> anyhow::Result<()> {
        // let mut text = String::new();
        // ctx.reader().read_to_string(&mut text)?;

        let config = TextureConfig {
            format: Format::RGBAFloat32,
            filtering: FilterMode::Linear,
            wrap_x: WrapMode::Repeat,
            wrap_y: WrapMode::Repeat,
            aniso_level: 0.0,
            generate_mips: true,
            ..Default::default()
        };

        let (data, width, height) = crate::image::open_hdr(ctx.reader())?;

        let texture = Texture2D::new(&self.loader.device, &data, width, height, config)?;

        ctx.set_asset(self.loader.load_from_hdr(texture)?);

        Ok(())
    }
}

#[derive(Clone, type_uuid::TypeUuid)]
#[uuid = "ab0e8f3d-9731-4435-9732-72e69143f8c7"]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default)
)]
pub struct Environment {
    pub texture: Option<Handle<EnvironmentTexture>>,
    pub intensity: f32,
    pub use_proxy: bool,
    pub mip_level: u32,
}
impl Default for Environment {
    fn default() -> Self {
        Self {
            texture: None,
            intensity: 1.0,
            use_proxy: false,
            mip_level: 0,
        }
    }
}

const CUBEMAP_DIM: u32 = 512;
const DIFF_IRRADIANCE_DIM: u32 = 64;
const SPECULAR_PF_DIM: u32 = 512;

pub struct HDRLoader {
    device: Arc<Device>,
    graph: Mutex<Graph<GraphParams>>,
}

impl HDRLoader {
    pub fn new(gfx: &mut Gfx, shader_lib: &mut ShaderLibrary) -> anyhow::Result<Self> {
        let graph = Self::create_hdr_loader_graph(gfx, shader_lib)?;
        Ok(Self {
            device: gfx.device().clone(),
            graph: Mutex::new(graph),
        })
    }
    pub fn load_from_hdr(&self, hdr_map: Texture2D) -> anyhow::Result<EnvironmentTexture> {
        let config = TextureConfig {
            format: Format::RGBAFloat16,
            filtering: FilterMode::Linear,
            wrap_x: WrapMode::Clamp,
            wrap_y: WrapMode::Clamp,
            aniso_level: 0.0,
            generate_mips: true,
            ..Default::default()
        };
        let mut vkconfig = config.into_image_config_cube(CUBEMAP_DIM, CUBEMAP_DIM)?;

        vkconfig.usage = vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::STORAGE;
        vkconfig.initial_layout = vk::ImageLayout::GENERAL;

        let skybox =
            SampledImage::with_dimensions(&self.device, CUBEMAP_DIM, CUBEMAP_DIM, 1, 6, vkconfig)?;

        let mut vkconfig =
            config.into_image_config_cube(DIFF_IRRADIANCE_DIM, DIFF_IRRADIANCE_DIM)?;

        vkconfig.usage = vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::STORAGE;
        vkconfig.initial_layout = vk::ImageLayout::GENERAL;

        let diffuse_irradiance = SampledImage::with_dimensions(
            &self.device,
            DIFF_IRRADIANCE_DIM,
            DIFF_IRRADIANCE_DIM,
            1,
            6,
            vkconfig,
        )?;

        let mut vkconfig = config.into_image_config_cube(SPECULAR_PF_DIM, SPECULAR_PF_DIM)?;

        vkconfig.usage = vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::STORAGE;
        vkconfig.initial_layout = vk::ImageLayout::GENERAL;

        let specular_prefiltered = SampledImage::with_dimensions(
            &self.device,
            SPECULAR_PF_DIM,
            SPECULAR_PF_DIM,
            1,
            6,
            vkconfig,
        )?;

        //let _frame = hikari_render::renderdoc::FrameCapture::new();

        self.graph.lock().execute_sync((
            &hdr_map.raw(),
            &skybox,
            &diffuse_irradiance,
            &specular_prefiltered,
        ))?;

        Ok(EnvironmentTexture {
            skybox,
            diffuse_irradiance,
            specular_prefiltered,
        })
    }
    pub fn create_hdr_loader_graph(
        gfx: &mut Gfx,
        shader_lib: &mut ShaderLibrary,
    ) -> anyhow::Result<Graph<GraphParams>> {
        let mut graph =
            GraphBuilder::<GraphParams>::new(gfx, DIFF_IRRADIANCE_DIM, DIFF_IRRADIANCE_DIM);

        shader_lib.insert("equiangular_to_cubemap")?;
        let shader = shader_lib.get("equiangular_to_cubemap").unwrap().clone();

        graph.add_computepass(ComputePass::new("EquiangularToCubemap").cmd(
            move |cmd,
                  _res,
                  _record,
                  (hdr_map, env_map, _diffuse_irr, _spec): (
                &SampledImage,
                &SampledImage,
                &SampledImage,
                &SampledImage,
            )| {
                cmd.set_shader(&shader);

                cmd.set_image(hdr_map, 1, 0);

                // let view_desc = ImageViewDesc {
                //     view_type: vk::ImageViewType::CUBE,
                //     mip_range: 0..1,
                //     layer_range: 0..6,
                // };

                //let image_view = diffuse_irr.raw().custom_image_view(view_desc);

                cmd.set_image(env_map, 1, 1);

                let work_groups = n_workgroups(CUBEMAP_DIM, 16);
                cmd.dispatch((work_groups, work_groups, 1));

                cmd.apply_image_barrier(
                    env_map,
                    &[AccessType::ComputeShaderWrite],
                    &[AccessType::TransferRead],
                    ImageLayout::Optimal,
                    ImageLayout::Optimal,
                    *vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(vk::REMAINING_MIP_LEVELS)
                        .base_array_layer(0)
                        .layer_count(vk::REMAINING_ARRAY_LAYERS),
                );

                env_map.generate_mips(cmd.raw());

                cmd.apply_image_barrier(
                    env_map,
                    &[AccessType::TransferRead],
                    &[AccessType::ComputeShaderReadSampledImageOrUniformTexelBuffer],
                    ImageLayout::Optimal,
                    ImageLayout::Optimal,
                    *vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(vk::REMAINING_MIP_LEVELS)
                        .base_array_layer(0)
                        .layer_count(vk::REMAINING_ARRAY_LAYERS),
                );
            },
        ));

        shader_lib.insert("cubemap_convolve")?;
        let shader_diffuse = shader_lib.get("cubemap_convolve").unwrap().clone();
        let shader_specular = shader_diffuse.clone();

        #[repr(C)]
        #[derive(Clone, Copy, Default)]
        struct PushConstants {
            convolve_type: u32,
            roughness: f32,
            _padding: [f32; 2],
        }
        const DIFFUSE_IRRADIANCE_CONVOLVE: u32 = 1;
        const SPECULAR_PREFILTER_CONVOLVE: u32 = 2;
        graph.add_computepass(ComputePass::new("DiffuseIrradiance").cmd(
            move |cmd,
                  _res,
                  _record,
                  (_hdr_map, env_map, diffuse_irr, _spec): (
                &SampledImage,
                &SampledImage,
                &SampledImage,
                &SampledImage,
            )| {
                cmd.set_shader(&shader_diffuse);

                cmd.set_image(env_map, 1, 0);

                let view_desc = ImageViewDesc {
                    view_type: vk::ImageViewType::CUBE,
                    aspect: vk::ImageAspectFlags::COLOR,
                    mip_range: 0..1,
                    layer_range: 0..6,
                };
                let image_view = diffuse_irr.custom_image_view(&view_desc);

                cmd.set_image_view_and_sampler(image_view, vk::Sampler::null(), 1, 1, 0);

                cmd.push_constants(
                    &PushConstants {
                        convolve_type: DIFFUSE_IRRADIANCE_CONVOLVE,
                        roughness: 0.0,
                        ..Default::default()
                    },
                    0,
                );

                let work_groups = n_workgroups(DIFF_IRRADIANCE_DIM, 16);
                cmd.dispatch((work_groups, work_groups, 1));
            },
        ));

        let mip_count = (SPECULAR_PF_DIM as f32).log2().floor() as u32 + 1;

        graph.add_computepass(ComputePass::new("SpecularPrefilter").cmd(
            move |cmd,
                  _res,
                  _record,
                  (_hdr_map, env_map, _diffuse_irr, spec): (
                &SampledImage,
                &SampledImage,
                &SampledImage,
                &SampledImage,
            )| {
                cmd.set_shader(&shader_specular);

                cmd.set_image(env_map, 1, 0);

                for i in 0..mip_count {
                    let view_desc = ImageViewDesc {
                        view_type: vk::ImageViewType::CUBE,
                        aspect: vk::ImageAspectFlags::COLOR,
                        mip_range: i..i + 1,
                        layer_range: 0..6,
                    };
                    let image_view = spec.custom_image_view(&view_desc);

                    cmd.set_image_view_and_sampler(image_view, vk::Sampler::null(), 1, 1, 0);

                    let roughness = i as f32 / ((mip_count - 1) as f32);
                    cmd.push_constants(
                        &PushConstants {
                            convolve_type: SPECULAR_PREFILTER_CONVOLVE,
                            roughness,
                            ..Default::default()
                        },
                        0,
                    );

                    let work_groups = n_workgroups(SPECULAR_PF_DIM, 16);
                    cmd.dispatch((work_groups, work_groups, 1));
                }
            },
        ));

        graph.add_computepass(ComputePass::new("HDRGenerationFinal").cmd(
            move |cmd,
                  _res,
                  _record,
                  (_hdr_map, env_map, diffuse_irr, spec): (
                &SampledImage,
                &SampledImage,
                &SampledImage,
                &SampledImage,
            )| {
                cmd.apply_image_barrier(
                    env_map,
                    &[AccessType::ComputeShaderReadSampledImageOrUniformTexelBuffer],
                    &[AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer],
                    ImageLayout::Optimal,
                    ImageLayout::Optimal,
                    *vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(vk::REMAINING_MIP_LEVELS)
                        .base_array_layer(0)
                        .layer_count(vk::REMAINING_ARRAY_LAYERS),
                );

                cmd.apply_image_barrier(
                    diffuse_irr,
                    &[AccessType::ComputeShaderWrite],
                    &[AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer],
                    ImageLayout::Optimal,
                    ImageLayout::Optimal,
                    *vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(vk::REMAINING_MIP_LEVELS)
                        .base_array_layer(0)
                        .layer_count(vk::REMAINING_ARRAY_LAYERS),
                );

                //diffuse_irr.raw().generate_mips(cmd.raw());

                cmd.apply_image_barrier(
                    spec,
                    &[AccessType::ComputeShaderWrite],
                    &[AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer],
                    ImageLayout::Optimal,
                    ImageLayout::Optimal,
                    *vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(vk::REMAINING_MIP_LEVELS)
                        .base_array_layer(0)
                        .layer_count(vk::REMAINING_ARRAY_LAYERS),
                );
            },
        ));

        Ok(graph.build()?)
    }
}
