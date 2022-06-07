pub mod camera;
pub mod error;
mod gltf;
pub mod image;
pub mod light;
pub mod material;
pub mod mesh;
pub mod scene;
pub mod texture;

use std::ffi::OsStr;

pub use camera::*;
pub use error::Error;
pub use light::*;
pub use material::*;
pub use mesh::*;
pub use scene::*;
pub use texture::*;

pub enum MeshFormat {
    Gltf,
    Fbx,
}
impl MeshFormat {
    pub fn from_extension(ext: &OsStr) -> Result<MeshFormat, crate::Error> {
        let ext_str = ext.to_str().unwrap().to_ascii_lowercase();
        match ext_str.as_str() {
            "fbx" => Ok(MeshFormat::Fbx),
            "gltf" | "glb" => Ok(MeshFormat::Gltf),
            _ => Err(crate::Error::UnsupportedModelFormat(ext.to_owned())),
        }
    }
}

pub mod old;