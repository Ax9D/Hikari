use std::path::Path;

use crate::{error, mesh::MeshFormat};
pub struct Scene {
    pub(crate) textures: Vec<crate::Texture>,
    pub(crate) materials: Vec<crate::Material>,
    pub(crate) models: Vec<crate::Model>,
}
impl Scene {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .ok_or(error::IOErrors::FailedToIdentifyFormat(
                path.as_os_str().to_owned(),
            ))?;

        let format = MeshFormat::from_extension(extension)?;

        match format {
            MeshFormat::Gltf => crate::gltf::load_scene(path),
            MeshFormat::Fbx => {
                todo!()
            }
        }
    }
    pub fn textures(&self) -> std::slice::Iter<'_, crate::Texture> {
        self.textures.iter()
    }
    pub fn materials(&self) -> std::slice::Iter<'_, crate::Material> {
        self.materials.iter()
    }
    pub fn models(&self) -> std::slice::Iter<'_, crate::Model> {
        self.models.iter()
    }
}
