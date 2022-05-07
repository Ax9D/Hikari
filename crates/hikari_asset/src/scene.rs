// use std::path::Path;

// use crate::{error, mesh::MeshFormat};
// pub struct Scene {
//     pub textures: Vec<crate::Texture>,
//     pub materials: Vec<crate::Material>,
//     pub models: Vec<crate::Model>,
// }
// impl Scene {
//     pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, crate::Error> {
//         let path = path.as_ref();
//         let extension = path
//             .extension()
//             .ok_or(error::Error::FailedToIdentifyFormat(
//                 path.as_os_str().to_owned(),
//             ))?;

//         let format = MeshFormat::from_extension(extension)?;

//         match format {
//             MeshFormat::Gltf => crate::gltf::load_scene(path),
//             MeshFormat::Fbx => {
//                 todo!()
//             }
//         }
//     }

// }
