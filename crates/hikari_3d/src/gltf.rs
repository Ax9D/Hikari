use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    sync::Arc,
};

use hikari_asset::{Handle, LoadContext, Mode};
use hikari_math::{Quat, Vec2, Vec3, Vec4};

use crate::{
    material::Material,
    texture::{Texture2D, TextureConfig},
    SubMesh,
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
// fn load_mesh_data(import_data: &ImportData, mesh: &gltf::Mesh<'_>) -> crate::mesh::Mesh {
//     let mut meshes = Vec::new();
//     let name = mesh
//         .name()
//         .unwrap_or(&format!(
//             "{}_model_{}",
//             import_data.filename(),
//             mesh.index()
//         ))
//         .to_owned();
//     //println!("Loading model {}", name);
//     for primitive in mesh.primitives() {
//         let reader = primitive.reader(|buffer| Some(&import_data.buffers()[buffer.index()]));

//         let positions: Vec<_> = if let Some(iter) = reader.read_positions() {
//             let positions = iter.collect::<Vec<_>>();
//             positions
//                 .iter()
//                 .map(|position| Vec3::from(*position))
//                 .collect()
//         } else {
//             continue;
//         };

//         let normals = if let Some(iter) = reader.read_normals() {
//             let normals = iter.collect::<Vec<_>>();
//             normals.iter().map(|normal| Vec3::from(*normal)).collect()
//         } else {
//             crate::mesh::default_normals(positions.len())
//         };

//         let texcoord0 = if let Some(iter) = reader.read_tex_coords(0) {
//             let iter = iter.into_f32();
//             let texcoord0 = iter.collect::<Vec<_>>();
//             texcoord0
//                 .iter()
//                 .map(|texcoord0| Vec2::from(*texcoord0))
//                 .collect()
//         } else {
//             vec![Vec2::ZERO; positions.len()]
//         };

//         let texcoord1 = if let Some(iter) = reader.read_tex_coords(1) {
//             let iter = iter.into_f32();
//             let texcoord1 = iter.collect::<Vec<_>>();
//             texcoord1
//                 .iter()
//                 .map(|texcoord1| Vec2::from(*texcoord1))
//                 .collect()
//         } else {
//             vec![Vec2::ZERO; positions.len()]
//         };

//         let indices = if let Some(iter) = reader.read_indices() {
//             let iter = iter.into_u32();
//             iter.collect::<Vec<_>>()
//         } else {
//             (0..positions.len()).map(|x| x as u32).collect::<Vec<_>>()
//         };
//         meshes.push(crate::mesh::SubMesh {
//             positions,
//             normals,
//             texcoord0,
//             texcoord1,
//             indices,
//             material: primitive.material().index(),
//         })
//     }

//     crate::mesh::Model { name, meshes }
// }
// fn load_meshes(import_data: &ImportData) -> Vec<crate::Mesh> {
//     // for mesh in importData.document().meshes() {
//     //     tokio::spawn(loadModelData(importData, mesh));
//     // }

//     import_data
//         .document()
//         .meshes()
//         .collect::<Vec<_>>()
//         .par_iter()
//         .map(|model| load_submeshes(&import_data, model))
//         .collect()

//     // for model in models {
//     //     for mesh in model {
//     //         crate::Me
//     //     }
//     // }
// }
// pub fn load_scene(path: &Path) -> Result<crate::Scene, crate::Error> {
//     let import_data = ImportData::new(path)
//         .map_err(|err| crate::Error::FailedToParse(path.into(), err.to_string()))?;

//     let now = std::time::Instant::now();
//     let textures = load_textures(&import_data)
//         .map_err(|err| crate::Error::FailedToParse(path.into(), err.to_string()))?;

//     //println!("Textures {:?}", now.elapsed());
//     //println!("First import texture {}", importData.document().textures().next().unwrap().index());
//     //println!("First texture {}", textures[0].name());

//     let now = std::time::Instant::now();
//     let materials = load_materials(&textures, &import_data);

//     let now = std::time::Instant::now();
//     //println!("Materials {:?}", now.elapsed());
//     let models = load_meshes(&import_data);

//     //println!("Models {:?}", now.elapsed());

//     Ok(crate::Scene {
//         textures,
//         materials,
//         models,
//     })
// }

fn load_textures(
    import_data: &ImportData,
    load_context: &mut LoadContext,
) -> Result<Vec<Handle<Texture2D>>, anyhow::Error> {
    // Ok(import_data
    //     .document()
    //     .textures()
    //     .collect::<Vec<_>>()
    //     .par_iter()
    //     .map(|texture| load_texture_data(&texture, &import_data).unwrap())
    //     .collect())
    let mut textures = Vec::new();
    for texture in import_data.document().textures() {
        textures.push(load_texture(import_data, &texture, load_context)?);
    }

    Ok(textures)
}
fn load_texture(
    import_data: &ImportData,
    texture: &gltf::Texture,
    load_context: &mut LoadContext,
) -> Result<Handle<Texture2D>, anyhow::Error> {
    let is_albedo = import_data.document().materials().find(|mat| {
        if let Some(albedo) = mat.pbr_metallic_roughness().base_color_texture() {
            albedo.texture().index() == texture.index()
        } else {
            false
        }
    });

    let wrap_x = match texture.sampler().wrap_s() {
        gltf::texture::WrappingMode::ClampToEdge => crate::texture::WrapMode::Clamp,
        gltf::texture::WrappingMode::MirroredRepeat => crate::texture::WrapMode::Repeat,
        gltf::texture::WrappingMode::Repeat => crate::texture::WrapMode::Repeat,
    };

    let wrap_y = match texture.sampler().wrap_t() {
        gltf::texture::WrappingMode::ClampToEdge => crate::texture::WrapMode::Clamp,
        gltf::texture::WrappingMode::MirroredRepeat => crate::texture::WrapMode::Repeat,
        gltf::texture::WrappingMode::Repeat => crate::texture::WrapMode::Repeat,
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
        gltf::texture::MagFilter::Nearest => crate::texture::FilterMode::Closest,
        gltf::texture::MagFilter::Linear => crate::texture::FilterMode::Linear,
    };

    let generate_mips = match min_filter {
        gltf::texture::MinFilter::NearestMipmapNearest
        | gltf::texture::MinFilter::NearestMipmapLinear
        | gltf::texture::MinFilter::LinearMipmapNearest
        | gltf::texture::MinFilter::LinearMipmapLinear => true,
        _ => false,
    } && is_albedo.is_some();
    //println!("Loading texture {}", name);

    //Albedo textures are treated as SRGB
    let format = if is_albedo.is_some() {
        crate::texture::Format::RGBA8
    } else {
        crate::texture::Format::RGBA8
    };

    let config = TextureConfig {
        format,
        filtering,
        wrap_x,
        wrap_y,
        aniso_level: 8.0,
        generate_mips,
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

            match mime_type {
                "image/png" | "image/jpeg" => create_and_load_image_with_data(
                    &data,
                    texture,
                    config,
                    mime_type,
                    gltf,
                    load_context,
                ),
                _ => Err(anyhow::anyhow!(
                    crate::error::Error::UnsupportedImageFormat(
                        mime_type.split(r"/").last().unwrap().to_string(),
                        texture.name().unwrap_or("unknown").to_string(),
                    )
                )),
            }
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

                match mime_type {
                    "image/png" | "image/jpeg" => create_and_load_image_with_data(
                        &data,
                        texture,
                        config,
                        mime_type,
                        gltf,
                        load_context,
                    ),
                    _ => Err(anyhow::anyhow!(
                        crate::error::Error::UnsupportedImageFormat(
                            mime_type.split(r"/").last().unwrap().to_string(),
                            texture.name().unwrap_or("unknown").to_string(),
                        )
                    )),
                }
            } else if let Some(mime_type) = mime_type {
                let path = gltf.parent_path().join(uri);

                match mime_type {
                    "image/jpeg" | "image/png" => asset_manager.load(&path, Some(config), false),
                    _ => Err(anyhow::anyhow!(
                        crate::error::Error::UnsupportedImageFormat(
                            mime_type.split(r"/").last().unwrap().to_string(),
                            texture.name().unwrap_or("unknown").to_string(),
                        )
                    )),
                }
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
    let texture_name = texture
        .name()
        .map(|name| name.to_owned())
        .unwrap_or_else(|| format!("{}_texture_{}", import_data.filename(), texture.index()));

    let ext = mime_type.split("/").nth(1).unwrap();

    let mut new_texture_path = base_path.to_owned();
    new_texture_path.push(texture_name);

    if new_texture_path.extension().is_none() {
        new_texture_path.set_extension(ext);
    }

    if !new_texture_path.exists() {
        let mut file = load_context
            .io()
            .write_file(&new_texture_path, &Mode::create_and_write())?;
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
    textures: &[Handle<Texture2D>],
    load_context: &mut LoadContext,
) -> Result<Vec<Handle<crate::Material>>, anyhow::Error> {
    let mut materials = Vec::new();
    for (ix, material) in import_data.document().materials().enumerate() {
        let material = load_material(ix, import_data, textures, &material, load_context)?;

        materials.push(material);
    }

    Ok(materials)
}
fn load_material(
    ix: usize,
    import_data: &ImportData,
    textures: &[Handle<Texture2D>],
    material: &gltf::Material,
    load_context: &mut LoadContext,
) -> Result<Handle<crate::Material>, anyhow::Error> {
    let mut file_name = material
        .name()
        .unwrap_or(&format!(
            "{}_material_{}",
            import_data.filename(),
            material.index().unwrap_or(ix)
        ))
        .to_owned();
    file_name.push_str(".hmat");

    let material_path = import_data.parent_path().join(file_name);
    if !material_path.exists() {
        let (albedo, albedo_set) =
            if let Some(info) = material.pbr_metallic_roughness().base_color_texture() {
                (
                    Some(textures[info.texture().index()].clone()),
                    info.tex_coord() as i32,
                )
            } else {
                (None, -1)
            };
        let albedo_factor = Vec4::from(material.pbr_metallic_roughness().base_color_factor());

        let (roughness, roughness_set) = if let Some(info) = material
            .pbr_metallic_roughness()
            .metallic_roughness_texture()
        {
            (
                Some(textures[info.texture().index()].clone()),
                info.tex_coord() as i32,
            )
        } else {
            (None, -1)
        };
        let roughness_factor = material.pbr_metallic_roughness().roughness_factor();

        let (metallic, metallic_set) = if let Some(info) = material
            .pbr_metallic_roughness()
            .metallic_roughness_texture()
        {
            (
                Some(textures[info.texture().index()].clone()),
                info.tex_coord() as i32,
            )
        } else {
            (None, -1)
        };
        let metallic_factor = material.pbr_metallic_roughness().metallic_factor();

        let (normal, normal_set) = if let Some(info) = material.normal_texture() {
            (
                Some(textures[info.texture().index()].clone()),
                info.tex_coord() as i32,
            )
        } else {
            (None, -1)
        };

        let material = Material {
            albedo,
            albedo_set,
            albedo_factor,
            roughness,
            roughness_set,
            roughness_factor,
            metallic,
            metallic_set,
            metallic_factor,
            normal,
            normal_set,
        };

        let material_text = serde_yaml::to_string(&material)?;
        std::fs::write(&material_path, material_text)?;
        println!("Creating material {ix} {:#?}", material_path);
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
    let _name = mesh
        .name()
        .unwrap_or(&format!("{}_mesh_{}", import_data.filename(), mesh.index()))
        .to_owned();

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

    let (positions, rotation, scale) = transform.decomposed();
    let transform = hikari_math::Transform {
        position: Vec3::from(positions),
        rotation: Quat::from_array(rotation),
        scale: Vec3::from(scale),
    };

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

        let indices = if let Some(iter) = reader.read_indices() {
            let iter = iter.into_u32();
            iter.collect::<Vec<_>>()
        } else {
            (0..positions.len()).map(|x| x as u32).collect::<Vec<_>>()
        };
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
    println!("Parsed GLTF");

    let textures = load_textures(&import_data, load_context)?;
    let materials = load_materials(&import_data, &textures, load_context)?;

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
            far: 10_000.0,
            exposure: 1.0,
            projection: crate::Projection::Perspective(45.0),
            is_primary: false,
        });

    Ok(crate::Scene { meshes, camera })
}
