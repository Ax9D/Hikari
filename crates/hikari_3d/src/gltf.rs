use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    sync::Arc,
};

use hikari_asset::{Handle, LoadContext, Mode};
use hikari_math::{Quat, Vec2, Vec3, Vec4};

use crate::{
    material::Material,
    texture::{Texture2D},

    SubMesh, TextureConfig, processing,
};
#[allow(unused)]
struct ImportData {
    path: PathBuf,
    parent_path: PathBuf,
    filename: String,
    document: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
}
impl ImportData {
    pub fn new(path: &Path, _data: &[u8]) -> Result<Self, gltf::Error> {
        assert!(path.is_relative());

        let (document, buffers, _images) = gltf::import(path)?;

        let parent_path = path.parent().unwrap_or_else(|| Path::new("./")).to_owned();
        Ok(Self {
            path: path.to_owned(),
            parent_path,
            filename: path
                .file_stem()
                .unwrap_or(&OsString::from("Unknown"))
                .to_str()
                .unwrap()
                .to_owned(),
            document,
            buffers,
        })
    }
    #[allow(unused)]
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn parent_path(&self) -> &Path {
        &self.parent_path
    }
    pub fn filename(&self) -> &str {
        &self.filename
    }
    pub fn document(&self) -> &gltf::Document {
        &self.document
    }
    pub fn buffers(&self) -> &Vec<gltf::buffer::Data> {
        &self.buffers
    }
    // pub fn images(&self) -> &Vec<gltf::image::Data> {
    //     &self.images
    // }
}
fn load_texture(
    import_data: &ImportData,
    texture: &gltf::Texture,
    load_context: &mut LoadContext,
    is_srgb: bool,
) -> Result<Handle<Texture2D>, anyhow::Error> {
    // let is_albedo = import_data.document().materials().find(|mat| {
    //     if let Some(albedo) = mat.pbr_metallic_roughness().base_color_texture() {
    //         albedo.texture().index() == texture.index()
    //     } else {
    //         false
    //     }
    // });

    let wrap_x = match texture.sampler().wrap_s() {
        gltf::texture::WrappingMode::ClampToEdge => crate::config::WrapMode::Clamp,
        gltf::texture::WrappingMode::MirroredRepeat => crate::config::WrapMode::Repeat,
        gltf::texture::WrappingMode::Repeat => crate::config::WrapMode::Repeat,
    };

    let wrap_y = match texture.sampler().wrap_t() {
        gltf::texture::WrappingMode::ClampToEdge => crate::config::WrapMode::Clamp,
        gltf::texture::WrappingMode::MirroredRepeat => crate::config::WrapMode::Repeat,
        gltf::texture::WrappingMode::Repeat => crate::config::WrapMode::Repeat,
    };

    let min_filter = texture
        .sampler()
        .min_filter()
        .unwrap_or(gltf::texture::MinFilter::Linear);
    let mag_filter = texture
        .sampler()
        .mag_filter()
        .unwrap_or(gltf::texture::MagFilter::Linear);

    let filtering = match mag_filter {
        gltf::texture::MagFilter::Nearest => crate::config::FilterMode::Closest,
        gltf::texture::MagFilter::Linear => crate::config::FilterMode::Linear,
    };

    let generate_mips = match min_filter {
        gltf::texture::MinFilter::NearestMipmapNearest
        | gltf::texture::MinFilter::NearestMipmapLinear
        | gltf::texture::MinFilter::LinearMipmapNearest
        | gltf::texture::MinFilter::LinearMipmapLinear => true,
        _=> false
    };
    //println!("Loading texture {}", name);

    // let is_albedo = import_data.document().materials().find(|material| {
    //     if let Some(info) = material.pbr_metallic_roughness().base_color_texture() {
    //         info.texture().index() == texture.index()
    //     } else {
    //         false
    //     }
    // }).is_some();

    //Albedo textures are treated as SRGB
    let format = if is_srgb {
        crate::config::Format::SRGBA
    } else {
        crate::config::Format::RGBA8
    };

    let config = TextureConfig {
        format,
        filtering,
        wrap_x,
        wrap_y,
        aniso_level: 8.0,
        generate_mips,
        ..Default::default()
    };

    parse_texture_data(texture, import_data, config, load_context)
}
fn parse_texture_data(
    texture: &gltf::Texture,
    gltf: &ImportData,
    config: TextureConfig,
    load_context: &mut LoadContext,
) -> Result<Handle<Texture2D>, anyhow::Error> {
    let asset_manager = load_context.asset_manager();
    match texture.source().source() {
        gltf::image::Source::View { view, mime_type } => {
            let start = view.offset();
            let end = start + view.length();

            let parent_buffer = &gltf.buffers()[view.buffer().index()].0;
            let data = &parent_buffer[start..end];

            create_and_load_image_with_data(
                    &data,
                    texture,
                    config,
                    mime_type,
                    gltf,
                    load_context,
            )

        }
        gltf::image::Source::Uri { uri, mime_type } => {
            //Credit: https://github.com/bwasty/gltf-viewer/blob/master/src/render/texture.rs

            if uri.starts_with("data:") {
                let encoded = uri.split(',').nth(1).unwrap();
                let data = base64::decode(&encoded).unwrap();
                let mime_type = if let Some(ty) = mime_type {
                    ty
                } else {
                    uri.split(',')
                        .nth(0)
                        .unwrap()
                        .split(':')
                        .nth(1)
                        .unwrap()
                        .split(';')
                        .nth(0)
                        .unwrap()
                };

                create_and_load_image_with_data(
                    &data,
                    texture,
                    config,
                    mime_type,
                    gltf,
                    load_context,
                )
            } else {
                let path = gltf.parent_path().join(uri);

                asset_manager.load(&path, Some(config), false)
            }
        }
    }
}
fn create_and_load_image_with_data(
    data: &[u8],
    texture: &gltf::Texture,
    config: TextureConfig,
    mime_type: &str,
    import_data: &ImportData,
    load_context: &mut LoadContext,
) -> anyhow::Result<Handle<Texture2D>> {
    let base_path = import_data.parent_path();

    let texture_id = texture.index().to_string();
    let texture_name = texture.name().unwrap_or(&texture_id);

    let texture_name = format!("{}_texture_{}", import_data.filename(), texture_name);

    let ext = mime_type.split("/").nth(1).unwrap();

    let mut new_texture_path = base_path.to_owned();
    new_texture_path.push(texture_name);

    if new_texture_path.extension().is_none() {
        new_texture_path.set_extension(ext);
    }

    let asset_dir = load_context.asset_dir();
    if !new_texture_path.exists() {
        let mut file = load_context
            .io()
            .write_file(&asset_dir.join(&new_texture_path), &Mode::create_and_write())?;
        file.write(&data)?;
        file.flush()?;
    }

    let texture =
        load_context
            .asset_manager()
            .load::<Texture2D>(new_texture_path, Some(config), false)?;

    Ok(texture)
}
fn load_materials(
    import_data: &ImportData,
    load_context: &mut LoadContext,
) -> Result<Vec<Handle<crate::Material>>, anyhow::Error> {
    let mut materials = Vec::new();
    for (ix, material) in import_data.document().materials().enumerate() {
        let material = load_material(ix, import_data, &material, load_context)?;

        materials.push(material);
    }

    Ok(materials)
}
fn load_material(
    ix: usize,
    import_data: &ImportData,
    material: &gltf::Material,
    load_context: &mut LoadContext,
) -> Result<Handle<crate::Material>, anyhow::Error> {
    let material_id = material.index().unwrap_or(ix).to_string();
    let material_name = material.name().unwrap_or(&material_id);

    let mut file_name = format!("{}_material_{}",import_data.filename(), material_name);

    file_name.push_str(".hmat");

    let material_path = import_data.parent_path().join(file_name);
    if !material_path.exists() {
        let pbr = material.pbr_metallic_roughness();

        let uv_set =pbr
        .base_color_texture()
        .or(pbr.metallic_roughness_texture())
        .map(|info| info.tex_coord())
        .unwrap_or(0);// Try to guess the uv_set

        let albedo = if let Some(info) = pbr.base_color_texture() {
            Some(load_texture(import_data, &info.texture(), load_context, true)?)
            } else {
                None
        };

        let albedo_factor = Vec4::from(pbr.base_color_factor());

        let roughness = if let Some(info) = pbr.metallic_roughness_texture() {
        Some(load_texture(import_data, &info.texture(), load_context, false)?)
        } else {
            None
        };

        let roughness_factor = pbr.roughness_factor();

        let metallic = if let Some(info) = pbr.metallic_roughness_texture() {
                Some(load_texture(import_data, &info.texture(), load_context, false)?)
            } else {
                None
            };

        let metallic_factor = pbr.metallic_factor();

        let emissive = if let Some(info) = material
            .emissive_texture() {
                Some(load_texture(import_data, &info.texture(), load_context, true)?)
            } else {
                None
            };


        let emissive_factor = material.emissive_factor().into();

        let normal = if let Some(info) = material
        .normal_texture() {
            Some(load_texture(import_data, &info.texture(), load_context, false)?)
        } else {
            None
        };


        let material = Material {
            albedo,
            uv_set,
            albedo_factor,
            roughness,
            roughness_factor,
            metallic,
            metallic_factor,
            normal,
            emissive,
            emissive_factor,
            ..Default::default()
        };

        let handle = load_context.asset_manager().create(&material_path, material, None)?;

        load_context.asset_manager().save(&handle)?;

        // let material_text = serde_yaml::to_string(&material)?;

        // let mut file = load_context.io().write_file(&material_path, &Mode::create_and_write())?;
        // file.write_all(material_text.as_bytes())?;

        //println!("Creating material {ix} {:#?}", material_path);
    }

    load_context
        .asset_manager()
        .load::<crate::Material>(&material_path, None, false)
}

fn load_mesh(
    import_data: &ImportData,
    device: &Arc<hikari_render::Device>,
    mesh: &gltf::Mesh<'_>,
    materials: &[Handle<crate::Material>],
) -> Result<crate::mesh::Mesh, anyhow::Error> {
    let mut sub_meshes = Vec::new();

    let node = import_data
        .document()
        .nodes()
        .find(|node| node.mesh().map(|mesh| mesh.index()) == Some(mesh.index()));

    let transform =
        node.map(|node| node.transform())
            .unwrap_or(gltf::scene::Transform::Decomposed {
                translation: [0.0, 0.0, 0.0],
                rotation: Quat::IDENTITY.to_array(),
                scale: [1.0, 1.0, 1.0],
            });

    let transform_matrix = hikari_math::Mat4::from_cols_array_2d(&transform.matrix());
    let mut transform = hikari_math::Transform::from_matrix(transform_matrix);
    transform = processing::left_handed_correction(transform);

    //println!("Loading model {}", name);
    for primitive in mesh.primitives() {
        let reader = primitive.reader(|buffer| Some(&import_data.buffers()[buffer.index()]));

        let positions: Vec<_> = if let Some(iter) = reader.read_positions() {
            let positions = iter.collect::<Vec<_>>();
            positions
                .iter()
                .map(|position| Vec3::from(*position))
                .collect()
        } else {
            continue;
        };

        let normals = if let Some(normals) = reader.read_normals() {
            normals.map(|normal| Vec3::from(normal)).collect()
        } else {
            crate::mesh::default_normals(positions.len())
        };

        let texcoord0 = if let Some(texcoord) = reader.read_tex_coords(0) {
            let texcoord = texcoord.into_f32();
            texcoord.map(|texcoord| Vec2::from(texcoord)).collect()
        } else {
            vec![Vec2::ZERO; positions.len()]
        };

        let texcoord1 = if let Some(texcoord) = reader.read_tex_coords(1) {
            let texcoord = texcoord.into_f32();
            texcoord.map(|texcoord| Vec2::from(texcoord)).collect()
        } else {
            vec![Vec2::ZERO; positions.len()]
        };

        let mut indices = if let Some(iter) = reader.read_indices() {
            let iter = iter.into_u32();
            iter.collect::<Vec<_>>()
        } else {
            (0..positions.len()).map(|x| x as u32).collect::<Vec<_>>()
        };

        //GLTF winding order is CCW
        //Change winding order to CW
        processing::ccw_to_cw(&mut indices);

        //let vertices = pack_for_gpu(positions, normals, texcoord0, texcoord1);
        let mut positions_buffer = hikari_render::create_vertex_buffer(device, positions.len())?;
        positions_buffer.upload(&positions, 0)?;

        let mut normals_buffer = hikari_render::create_vertex_buffer(device, normals.len())?;
        normals_buffer.upload(&normals, 0)?;

        let mut tc0_buffer = hikari_render::create_vertex_buffer(device, texcoord0.len())?;
        tc0_buffer.upload(&texcoord0, 0)?;

        let mut tc1_buffer = hikari_render::create_vertex_buffer(device, texcoord1.len())?;
        tc1_buffer.upload(&texcoord1, 0)?;

        let mut ibuffer = hikari_render::create_index_buffer(device, indices.len())?;
        ibuffer.upload(&indices, 0)?;

        let submesh = SubMesh {
            position: positions_buffer,
            normals: normals_buffer,
            tc0: tc0_buffer,
            tc1: tc1_buffer,
            indices: ibuffer,
            material: materials[primitive
                .material()
                .index()
                .expect("TODO: Handle default material")]
            .clone(),
        };

        sub_meshes.push(submesh);
    }

    Ok(crate::Mesh {
        sub_meshes,
        transform,
    })
}

pub fn load_scene(
    device: &Arc<hikari_render::Device>,
    path: &Path,
    data: &[u8],
    load_context: &mut LoadContext,
) -> Result<crate::Scene, anyhow::Error> {
    let import_data = ImportData::new(path, data)
        .map_err(|err| crate::Error::FailedToParse(path.into(), err.to_string()))?;

    //let textures = load_textures(&import_data, load_context)?;
    let materials = load_materials(&import_data, load_context)?;

    let mut meshes = Vec::new();

    for mesh in import_data.document().meshes() {
        let mesh = load_mesh(&import_data, device, &mesh, &materials)?;
        meshes.push(mesh);
    }

    let camera = import_data
        .document()
        .cameras()
        .next()
        .map(|camera| match camera.projection() {
            gltf::camera::Projection::Orthographic(ortho) => crate::Camera {
                near: ortho.znear(),
                far: ortho.zfar(),
                exposure: 1.0,
                projection: crate::Projection::Orthographic,
                is_primary: false,
            },
            gltf::camera::Projection::Perspective(persp) => crate::Camera {
                near: persp.znear(),
                far: persp.zfar().unwrap_or(1000.0),
                exposure: 1.0,
                projection: crate::Projection::Perspective(persp.yfov()),
                is_primary: false,
            },
        })
        .unwrap_or(crate::Camera {
            near: 0.1,
            far: 1000.0,
            exposure: 1.0,
            projection: crate::Projection::Perspective(45.0),
            is_primary: false,
        });

    Ok(crate::Scene { meshes, camera })
}
