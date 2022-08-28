pub mod camera;
pub mod error;
mod gltf;
pub mod image;
pub mod light;
pub mod material;
pub mod mesh;
pub mod scene;
pub mod texture;
mod shader;

use std::ffi::OsStr;

pub use camera::*;
pub use error::Error;
use hikari_asset::AssetManager;
use hikari_core::Plugin;
use hikari_render::Gfx;
pub use light::*;
pub use material::*;
pub use mesh::*;
pub use scene::*;
pub use texture::*;
pub use shader::*;

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

pub struct Plugin3D;

impl Plugin for Plugin3D {
    fn build(self, game: &mut hikari_core::Game) {
        game.create_asset::<Texture2D>();
        game.create_asset::<Material>();
        game.create_asset::<Scene>();

        let device = game.get::<Gfx>().device().clone();
        let shader_lib = ShaderLibrary::new(&device, std::env::current_dir().unwrap().join("assets/shaders"));
        game.add_state(shader_lib);

        let mut manager = game.get_mut::<AssetManager>();
        manager.add_loader::<Texture2D, TextureLoader>(TextureLoader {
            device: device.clone(),
        });
        manager.add_loader::<Material, MaterialLoader>(MaterialLoader);
        manager.add_loader::<Scene, GLTFLoader>(GLTFLoader {
            device,
        });
    }
}
