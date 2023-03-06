mod gltf;
mod processing;
mod shader;

pub mod camera;
pub mod config;
pub mod cubemap;
pub mod environment;
pub mod error;
pub mod image;
pub mod light;
pub mod material;
pub mod mesh;
pub mod primitives;
pub mod scene;
pub mod texture;

pub use camera::*;
pub use config::*;
pub use cubemap::*;
pub use environment::*;
pub use error::Error;
use hikari_core::Plugin;
use hikari_render::Gfx;
pub use light::*;
pub use material::*;
pub use mesh::*;
pub use scene::*;
pub use shader::*;
pub use texture::*;

pub mod old;

pub struct Plugin3D;

impl Plugin for Plugin3D {
    fn build(self, game: &mut hikari_core::Game) {
        game.create_asset::<Texture2D>();
        game.create_asset::<Material>();
        game.create_asset::<Scene>();
        game.create_asset::<EnvironmentTexture>();

        let device = {
            let gfx = game.get::<Gfx>();
            gfx.device().clone()
        };

        #[cfg(debug_assertions)]
        let generate_debug_info = true;
        #[cfg(not(debug_assertions))]
        let generate_debug_info = false;

        let config = ShaderLibraryConfig {
            generate_debug_info,
        };

        #[cfg(debug_assertions)]
        let base_path = std::path::Path::new("./");
        #[cfg(not(debug_assertions))]
        let base_path = hikari_utils::engine_dir();

        let mut shader_lib =
            ShaderLibrary::new(&device, base_path.join("data/assets/shaders"), config);
        let mut gfx = game.get_mut::<Gfx>();
        let primitives = primitives::Primitives::prepare(&mut gfx, &mut shader_lib);
        let env_loader = EnvironmentTextureLoader::new(&mut gfx, &mut shader_lib)
            .expect("Failed to create HDR Loader");
        drop(gfx);

        game.add_state(shader_lib);
        game.add_state(primitives);
        game.register_asset_loader::<Texture2D, TextureLoader>(TextureLoader {
            device: device.clone(),
        });
        game.register_asset_loader::<Material, MaterialLoader>(MaterialLoader);
        game.register_asset_saver::<Material, MaterialLoader>(MaterialLoader);

        game.register_asset_loader::<Scene, GLTFLoader>(GLTFLoader {
            device: device.clone(),
        });
        game.register_asset_loader::<EnvironmentTexture, EnvironmentTextureLoader>(env_loader);
    }
}
