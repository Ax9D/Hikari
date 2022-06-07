use std::path::Path;

use super::{error, mesh::MeshFormat};
pub struct Scene {
    pub textures: Vec<super::Texture>,
    pub materials: Vec<super::Material>,
    pub models: Vec<super::Model>,
}
impl Scene {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, super::Error> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .ok_or(error::Error::FailedToIdentifyFormat(
                path.as_os_str().to_owned(),
            ))?;

        let format = MeshFormat::from_extension(extension)?;

        match format {
            MeshFormat::Gltf => super::gltf::load_scene(path),
            MeshFormat::Fbx => {
                todo!()
            }
        }
    }

}
