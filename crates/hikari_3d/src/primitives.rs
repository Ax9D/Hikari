use std::{sync::Arc};

use hikari_math::Vec3;
use hikari_render::{Device, GpuBuffer, vk::{self}, SampledImage, GraphBuilder, AccessType, n_workgroups, ComputePass, vk_sync::ImageLayout, ImageConfig};

use crate::*;

pub struct Primitives {
    pub default_mat: Material,
    pub checkerboard: Texture2D,
    pub black: Texture2D,
    pub brdf_lut: SampledImage,
    pub black_cube: TextureCube,
    pub cube: Cube
}
impl Primitives {
    pub fn prepare(gfx: &mut Gfx, shader_lib: &mut ShaderLibrary) -> Arc<Self> {
        let device = &gfx.device().clone();

        let (checkerboard, width, height) =
            image::open_rgba8(hikari_utils::engine_dir().join("data/assets/textures/checkerboard.png"))
                .expect("Failed to load checkerboard texture");
        let checkerboard = Texture2D::new(
            device,
            &checkerboard,
            width,
            height,
            TextureConfig {
                format: Format::RGBA8,
                wrap_x: WrapMode::Repeat,
                wrap_y: WrapMode::Repeat,
                filtering: FilterMode::Linear,
                aniso_level: 9.0,
                generate_mips: true,
                ..Default::default()
            },
        )
        .expect("Failed to create checkerboard texture");

        let black = Texture2D::new(
            device,
            &[0_u8, 0, 0, 255],
            1,
            1,
            TextureConfig {
                format: Format::RGBA8,
                wrap_x: WrapMode::Repeat,
                wrap_y: WrapMode::Repeat,
                filtering: FilterMode::Linear,
                aniso_level: 0.0,
                generate_mips: false,
                ..Default::default()
            },
        )
        .expect("Failed to create black texture");

    //let (data, width, height) = image::open_rgba8("data/engine_assets/textures/brdf_lut.png").unwrap();
    // let (brdf_lut, width, height) = image::open_rgba32f("data/engine_assets/textures/brdf_lut.png")
    // .expect("Failed to load BRDF LUT texture");

        // let brdf_lut = Texture2D::new(
        //     device,
        //     &data,
        //     width,
        //     height,
        //     TextureConfig {
        //         format: Format::RGBA8,
        //         wrap_x: WrapMode::Clamp,
        //         wrap_y: WrapMode::Clamp,
        //         filtering: FilterMode::Linear,
        //         aniso_level: 0.0,
        //         generate_mips: false,
        //     },
        // )
        // .expect("Failed to create black texture");

        let brdf_lut = generate_brdf_lut(gfx, shader_lib).expect("Failed to generate BRDFLut");

        let black_cube = TextureCube::new(device,
            &[0_u8, 0, 0, 1].repeat(6),
            1,
            1,
            TextureConfig {
                format: Format::RGBA8,
                wrap_x: WrapMode::Repeat,
                wrap_y: WrapMode::Repeat,
                filtering: FilterMode::Linear,
                aniso_level: 0.0,
                generate_mips: false,
                ..Default::default()
            }).expect("Failed to create black texture");

        let default_mat = Material::default();

        let cube = load_cube(device).expect("Failed to load the mighty cube");

        Arc::new(Self {
            black,
            black_cube,
            checkerboard,
            brdf_lut,
            default_mat,
            cube,
        })
    }
}

pub struct Cube {
    pub verts: GpuBuffer<Vec3>,
    pub inds: GpuBuffer<u32>
}
fn load_cube(device: &Arc<Device>) -> anyhow::Result<Cube> {
    let cube = crate::old::Scene::load(hikari_utils::engine_dir().join("data/assets/models/cube.glb"))?;
    let cube = &cube.models[0].meshes[0];
    let mut verts = hikari_render::create_vertex_buffer(device, cube.positions.len())?;
    let mut inds = hikari_render::create_index_buffer(device, cube.indices.len())?;
    verts.upload(&cube.positions, 0)?;
    inds.upload(&cube.indices, 0)?;

    Ok(Cube {
        verts,
        inds
    })
}


fn generate_brdf_lut(gfx: &mut Gfx, shader_lib: &mut ShaderLibrary) -> anyhow::Result<SampledImage> {
    const LUT_SIZE: u32 = 512;
    const LOCAL_GROUP_SIZE: u32 = 16;

    let config = ImageConfig{
        format:  vk::Format::R16G16_SFLOAT,
        wrap_x:  vk::SamplerAddressMode::CLAMP_TO_EDGE,
        wrap_y:  vk::SamplerAddressMode::CLAMP_TO_EDGE,
        wrap_z:  vk::SamplerAddressMode::CLAMP_TO_EDGE,
        usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::STORAGE,
        initial_layout: vk::ImageLayout::GENERAL,

        ..Default::default()
    };

    let img = SampledImage::with_dimensions(gfx.device(), LUT_SIZE, LUT_SIZE, 1, 1, config)?;

    shader_lib.insert("generate_brdf_lut")?;
    let shader = shader_lib.get("generate_brdf_lut").unwrap().clone();

    let mut graph = GraphBuilder::<(SampledImage,)>::new(gfx, LUT_SIZE, LUT_SIZE);
    graph.add_computepass(ComputePass::new("BRDFLUTGen")
    .cmd(move |cmd, _, _, (img,)| {
        cmd.set_shader(&shader);

        cmd.set_image(img, 0, 0);

        let workgroups = n_workgroups(LUT_SIZE, LOCAL_GROUP_SIZE);

        cmd.dispatch((workgroups, workgroups, 1));

        cmd.apply_image_barrier(img, &[AccessType::ComputeShaderWrite],
        &[AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer],
        ImageLayout::Optimal,
        ImageLayout::Optimal,
        *vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(vk::REMAINING_MIP_LEVELS)
        .base_array_layer(0)
        .layer_count(vk::REMAINING_ARRAY_LAYERS)
        );
    }));

    let mut graph = graph.build()?;

    let now = std::time::Instant::now();

    graph.execute_sync((&img,))?;

    log::debug!("Generated BRDF LUT in {:?}", now.elapsed());

    Ok(img)
}