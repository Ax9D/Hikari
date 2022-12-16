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

        #[cfg(debug_assertions)]
        let generate_debug_info = true;
        #[cfg(not(debug_assertions))]
        let generate_debug_info = false;

        let config = ShaderLibraryConfig {
            generate_debug_info
        };

        let shader_lib = ShaderLibrary::new(&device, std::env::current_dir().unwrap().join("engine_assets/shaders"), config);
        game.add_state(shader_lib);
        game.register_asset_loader::<Texture2D, TextureLoader>(TextureLoader {
            device: device.clone(),
        });
        game.register_asset_loader::<Material, MaterialLoader>(MaterialLoader);
        game.register_asset_loader::<Scene, GLTFLoader>(GLTFLoader {
            device,
        });
    }
}
