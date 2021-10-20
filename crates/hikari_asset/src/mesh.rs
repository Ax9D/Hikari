use std::{ffi::OsStr, path::Path};

use crate::error;

// impl Mesh {
//     pub fn new() -> Arc<Mesh> {
//     }
// }

pub enum MeshFormat {
    Gltf,
    Fbx,
}
impl MeshFormat {
    pub fn from_extension(ext: &OsStr) -> Result<MeshFormat, error::IOErrors> {
        let ext_str = ext.to_str().unwrap().to_ascii_lowercase();
        match ext_str.as_str() {
            "fbx" => Ok(MeshFormat::Fbx),
            "gltf" | "glb" => Ok(MeshFormat::Gltf),
            _ => Err(error::IOErrors::UnsupportedModelFormat(ext.to_owned())),
        }
    }
}
#[derive(Debug)]
pub struct SceneData {
    positions: Vec<f32>,
    indices: Vec<u32>,
    tex_coords: Vec<f32>,
}
#[derive(Debug)]
pub struct MeshData {
    positions: Vec<f32>,
    indices: Vec<u32>,
    tex_coords: Vec<f32>,
    normals: Vec<f32>,
}
impl MeshData {
    pub fn positions(&self) -> &Vec<f32> {
        &self.positions
    }
    pub fn indices(&self) -> &Vec<u32> {
        &self.indices
    }
    pub fn tex_coords(&self) -> &Vec<f32> {
        &self.tex_coords
    }
    pub fn normals(&self) -> &Vec<f32> {
        &self.normals
    }
}
pub struct Mesh {
    pub(crate) positions: Vec<glam::Vec3>,
    pub(crate) normals: Vec<glam::Vec3>,
    pub(crate) texcoord0: Vec<glam::Vec2>,
    pub(crate) texcoord1: Vec<glam::Vec2>,

    pub(crate) indices: Vec<u32>,

    pub(crate) material: Option<usize>,
}
impl Mesh {
    pub fn positions(&self) -> &[glam::Vec3] {
        &self.positions
    }
    pub fn normals(&self) -> &[glam::Vec3] {
        &self.normals
    }
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn texcoord0(&self) -> &[glam::Vec2] {
        &self.texcoord0
    }
    pub fn texcoord1(&self) -> &[glam::Vec2] {
        &self.texcoord1
    }
    pub fn material(&self) -> Option<usize> {
        self.material
    }
}
pub struct Model {
    pub(crate) name: String,
    pub(crate) meshes: Vec<Mesh>,
}

impl Model {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn meshes(&self) -> std::slice::Iter<Mesh> {
        self.meshes.iter()
    }
}
pub fn default_normals(n: usize) -> Vec<glam::Vec3> {
    //Flat normals
    vec![glam::Vec3::ZERO; n]
}

impl MeshData {
    fn process_node() {}
    // fn loadFromFile_(path: &Path) -> Result<Model, Box<dyn std::error::Error>> {
    //     use russimp::*;

    //     let scene = scene::Scene::from_file(path.as_os_str().to_str().unwrap(), vec![PostProcess::Triangulate, PostProcess::FlipUVs, PostProcess::CalculateTangentSpace])?;

    //     scene.root.unwrap().borrow()
    // }

    #[deprecated]
    fn load_gltf(path: &Path) -> Result<MeshData, Box<dyn std::error::Error>> {
        let scene = easy_gltf::load(path)?.remove(0);
        let vertices = scene.models[0].vertices();

        let mut positions = Vec::new();
        let mut tex_coords = Vec::new();
        let mut normals = Vec::new();

        for vertex in vertices {
            positions.push(vertex.position.x);
            positions.push(vertex.position.y);
            positions.push(vertex.position.z);

            tex_coords.push(vertex.tex_coords.x);
            tex_coords.push(vertex.tex_coords.y);

            normals.push(vertex.normal.x);
            normals.push(vertex.normal.y);
            normals.push(vertex.normal.z);
        }

        let indices: Vec<u32> = scene.models[0]
            .indices()
            .unwrap()
            .iter()
            .map(|x| *x as u32)
            .collect();

        Ok(MeshData {
            positions,
            tex_coords,
            indices,
            normals,
        })
    }
    pub fn load_from_file<P: AsRef<str>>(path: P) -> Result<MeshData, Box<dyn std::error::Error>> {
        let path = Path::new(path.as_ref());

        let extension = path
            .extension()
            .ok_or(error::IOErrors::FailedToIdentifyFormat(
                path.as_os_str().to_owned(),
            ))?;

        let format = MeshFormat::from_extension(extension)?;

        match format {
            MeshFormat::Gltf => Self::load_gltf(path),
            MeshFormat::Fbx => {
                todo!()
            }
        }
    }
}
