use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use rayon::iter::*;
use rayon::*;
use hikari_math::{Vec2, Vec3, Vec4};

use crate::{
    material::{Material, TextureDesc},
    texture::Texture,
};
struct ImportData {
    path: PathBuf,
    filename: String,
    document: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
    images: Vec<gltf::image::Data>,
}
impl ImportData {
    pub fn new(path: &Path) -> Result<Self, gltf::Error> {
        let (document, buffers, images) = gltf::import(path)?;
        Ok(Self {
            path: path.to_owned(),
            filename: path
                .file_stem()
                .unwrap_or(&OsString::from("Unknown"))
                .to_str()
                .unwrap()
                .to_owned(),
            document,
            buffers,
            images,
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
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
    pub fn images(&self) -> &Vec<gltf::image::Data> {
        &self.images
    }
}
fn parse_texture_data(
    texture: &gltf::Texture,
    gltf: &ImportData,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    Ok(match texture.source().source() {
        gltf::image::Source::View { view, mime_type } => {
            let start = view.offset();
            let end = start + view.length();

            let parent_buffer = &gltf.buffers()[view.buffer().index()].0;
            let data = &parent_buffer[start..end];

            match mime_type {
                "image/jpeg" => Ok(crate::image::load_from_data(
                    data,
                    image::ImageFormat::Jpeg,
                )?),
                "image/png" => Ok(crate::image::load_from_data(data, image::ImageFormat::Png)?),
                _ => Err(crate::error::Error::UnsupportedImageFormat(
                    mime_type.split(r"/").last().unwrap().to_string(),
                    texture.name().unwrap_or("unknown").to_string(),
                )),
            }?
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
                    "image/jpeg" => Ok(crate::image::load_from_data(
                        &data,
                        image::ImageFormat::Jpeg,
                    )?),
                    "image/png" => Ok(crate::image::load_from_data(
                        &data,
                        image::ImageFormat::Png,
                    )?),
                    _ => Err(crate::error::Error::UnsupportedImageFormat(
                        mime_type.split(r"/").last().unwrap().to_string(),
                        texture.name().unwrap_or("unknown").to_string(),
                    )),
                }?
            } else if let Some(mime_type) = mime_type {
                let path = gltf
                    .path()
                    .parent()
                    .unwrap_or_else(|| Path::new("./"))
                    .join(uri);

                match mime_type {
                    "image/jpeg" => Ok(crate::image::load_from_file_with_format(
                        &path,
                        image::ImageFormat::Jpeg,
                    )?),
                    "image/png" => Ok(crate::image::load_from_file_with_format(
                        &path,
                        image::ImageFormat::Png,
                    )?),
                    _ => Err(crate::error::Error::UnsupportedImageFormat(
                        mime_type.split(r"/").last().unwrap().to_string(),
                        texture.name().unwrap_or("unknown").to_string(),
                    )),
                }?
            } else {
                let path = gltf
                    .path()
                    .parent()
                    .unwrap_or_else(|| Path::new("./"))
                    .join(uri);
                crate::image::load_from_file(&path)?
            }
        }
    })
}

fn load_texture_data(
    texture: &gltf::Texture,
    gltf: &ImportData,
) -> Result<Texture, Box<dyn std::error::Error>> {
    let (data, width, height) = parse_texture_data(texture, gltf)?;

    let is_albedo = gltf.document().materials().find(|mat| {
        if let Some(albedo) = mat.pbr_metallic_roughness().base_color_texture() {
            albedo.texture().index() == texture.index()
        } else {
            false
        }
    });

    let wrap_x = match texture.sampler().wrap_s() {
        gltf::texture::WrappingMode::ClampToEdge => hikari_3d::texture::WrapMode::Clamp,
        gltf::texture::WrappingMode::MirroredRepeat => hikari_3d::texture::WrapMode::Repeat,
        gltf::texture::WrappingMode::Repeat => hikari_3d::texture::WrapMode::Repeat,
    };

    let wrap_y = match texture.sampler().wrap_t() {
        gltf::texture::WrappingMode::ClampToEdge => hikari_3d::texture::WrapMode::Clamp,
        gltf::texture::WrappingMode::MirroredRepeat => hikari_3d::texture::WrapMode::Repeat,
        gltf::texture::WrappingMode::Repeat => hikari_3d::texture::WrapMode::Repeat,
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
        gltf::texture::MagFilter::Nearest => hikari_3d::texture::FilterMode::Closest,
        gltf::texture::MagFilter::Linear => hikari_3d::texture::FilterMode::Linear,
    };

    let generate_mips = match min_filter {
        gltf::texture::MinFilter::NearestMipmapNearest
        | gltf::texture::MinFilter::NearestMipmapLinear
        | gltf::texture::MinFilter::LinearMipmapNearest
        | gltf::texture::MinFilter::LinearMipmapLinear => true,
        _ => false,
    } && is_albedo.is_some();

    let name = texture
        .name()
        .unwrap_or(&format!("{}_texture_{}", gltf.filename(), texture.index()))
        .to_owned();

    //println!("Loading texture {}", name);

    //Albedo textures are treated as SRGB
    let format = if is_albedo.is_some() {
        hikari_3d::texture::Format::RGBA8
    } else {
        hikari_3d::texture::Format::RGBA8
    };

    Ok(Texture {
        name,
        width,
        height,
        data,
        filtering,
        generate_mips,
        wrap_x,
        wrap_y,
        format,
    })
}
fn load_textures(import_data: &ImportData) -> Result<Vec<Texture>, Box<dyn std::error::Error>> {
    use rayon::prelude::*;
    Ok(import_data
        .document()
        .textures()
        .collect::<Vec<_>>()
        .par_iter()
        .map(|texture| load_texture_data(&texture, &import_data).unwrap())
        .collect())
}
fn load_materials(textures: &Vec<Texture>, import_data: &ImportData) -> Vec<Material> {
    let mut ret = Vec::new();

    for material in import_data.document().materials() {
        // let albedo = if let Some(textureInfo) = material.pbr_metallic_roughness().base_color_texture() {
        //     MaterialColor::Texture(textures.get(&textureInfo.texture().index()).expect("Null").clone())
        // } else {
        //     MaterialColor::Constant(glam::Vec4::from(material.pbr_metallic_roughness().base_color_factor()))
        // };

        // let roughness = if let Some(textureInfo) = material.pbr_metallic_roughness().metallic_roughness_texture() {
        //     MaterialValue::Texture(textures.get(&textureInfo.texture().index()).expect("Null").clone())
        // } else {
        //     MaterialValue::Constant(material.pbr_metallic_roughness().roughness_factor())
        // };
        // let metallic = if let Some(textureInfo) = material.pbr_metallic_roughness().metallic_roughness_texture() {
        //     MaterialValue::Texture(textures.get(&textureInfo.texture().index()).expect("Null").clone())
        // } else {
        //     MaterialValue::Constant(material.pbr_metallic_roughness().metallic_factor())
        // };

        let albedo_map = if let Some(info) = material.pbr_metallic_roughness().base_color_texture()
        {
            Some(TextureDesc {
                index: info.texture().index(),
                tex_coord_set: info.tex_coord(),
            })
        } else {
            None
        };
        let albedo = Vec4::from(material.pbr_metallic_roughness().base_color_factor());

        let roughness_map = if let Some(info) = material
            .pbr_metallic_roughness()
            .metallic_roughness_texture()
        {
            Some(TextureDesc {
                index: info.texture().index(),
                tex_coord_set: info.tex_coord(),
            })
        } else {
            None
        };
        let roughness = material.pbr_metallic_roughness().roughness_factor();

        let metallic_map = if let Some(info) = material
            .pbr_metallic_roughness()
            .metallic_roughness_texture()
        {
            Some(TextureDesc {
                index: info.texture().index(),
                tex_coord_set: info.tex_coord(),
            })
        } else {
            None
        };
        let metallic = material.pbr_metallic_roughness().metallic_factor();

        let normal_map = if let Some(info) = material.normal_texture() {
            Some(TextureDesc {
                index: info.texture().index(),
                tex_coord_set: info.tex_coord(),
            })
        } else {
            None
        };
        let name = material
            .name()
            .unwrap_or(&format!(
                "{}_material_{}",
                import_data.filename(),
                material.index().unwrap_or(ret.len())
            ))
            .to_owned();

        ret.push(crate::material::Material {
            name,
            albedo,
            albedo_map,
            metallic,
            metallic_map,
            roughness,
            roughness_map,
            normal_map,
        });
    }
    ret
}
fn load_model_data(import_data: &ImportData, mesh: &gltf::Mesh<'_>) -> crate::mesh::Model {
    let mut meshes = Vec::new();
    let name = mesh
        .name()
        .unwrap_or(&format!(
            "{}_model_{}",
            import_data.filename(),
            mesh.index()
        ))
        .to_owned();
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

        let normals = if let Some(iter) = reader.read_normals() {
            let normals = iter.collect::<Vec<_>>();
            normals
                .iter()
                .map(|normal| Vec3::from(*normal))
                .collect()
        } else {
            crate::mesh::default_normals(positions.len())
        };

        let texcoord0 = if let Some(iter) = reader.read_tex_coords(0) {
            let iter = iter.into_f32();
            let texcoord0 = iter.collect::<Vec<_>>();
            texcoord0
                .iter()
                .map(|texcoord0| Vec2::from(*texcoord0))
                .collect()
        } else {
            vec![Vec2::ZERO; positions.len()]
        };

        let texcoord1 = if let Some(iter) = reader.read_tex_coords(0) {
            let iter = iter.into_f32();
            let texcoord1 = iter.collect::<Vec<_>>();
            texcoord1
                .iter()
                .map(|texcoord1| Vec2::from(*texcoord1))
                .collect()
        } else {
            vec![Vec2::ZERO; positions.len()]
        };

        let indices = if let Some(iter) = reader.read_indices() {
            let iter = iter.into_u32();
            iter.collect::<Vec<_>>()
        } else {
            (0..positions.len()).map(|x| x as u32).collect::<Vec<_>>()
        };
        meshes.push(crate::mesh::Mesh {
            positions,
            normals,
            texcoord0,
            texcoord1,
            indices,
            material: primitive.material().index(),
        })
    }

    crate::mesh::Model { name, meshes }
}
fn load_models(import_data: &ImportData) -> Vec<crate::mesh::Model> {
    // for mesh in importData.document().meshes() {
    //     tokio::spawn(loadModelData(importData, mesh));
    // }

    import_data
        .document()
        .meshes()
        .collect::<Vec<_>>()
        .par_iter()
        .map(|model| load_model_data(&import_data, model))
        .collect()

    // for model in models {
    //     for mesh in model {
    //         crate::Me
    //     }
    // }
}
pub fn load_scene(path: &Path) -> Result<crate::Scene, crate::Error> {
    let import_data = ImportData::new(path)
        .map_err(|err| crate::Error::FailedToParse(path.into(), err.to_string()))?;

    let now = std::time::Instant::now();
    let textures = load_textures(&import_data)
        .map_err(|err| crate::Error::FailedToParse(path.into(), err.to_string()))?;

    //println!("Textures {:?}", now.elapsed());
    //println!("First import texture {}", importData.document().textures().next().unwrap().index());
    //println!("First texture {}", textures[0].name());

    let now = std::time::Instant::now();
    let materials = load_materials(&textures, &import_data);

    let now = std::time::Instant::now();
    //println!("Materials {:?}", now.elapsed());
    let models = load_models(&import_data);

    //println!("Models {:?}", now.elapsed());

    Ok(crate::Scene {
        textures,
        materials,
        models,
    })
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn godzilla() {
        let model = gltf::load_scene("godzilla.glb".as_ref()).unwrap();
    }
}
