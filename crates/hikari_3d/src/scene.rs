use std::sync::Arc;

use hikari_asset::{Asset, LoadContext, Loader};

use crate::{Camera, Mesh};

#[derive(type_uuid::TypeUuid)]
#[uuid = "90eff7a8-4a6b-444f-bc09-dbc441bda057"]
pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub camera: Camera,
}
pub struct GLTFLoader {
    pub device: Arc<hikari_render::Device>,
}
impl Loader for GLTFLoader {
    fn load(&self, context: &mut LoadContext) -> anyhow::Result<()> {
        // let mut data = vec![];
        // context.reader().read_to_end(&mut data)?;
        let path = context.path().to_owned();
        let scene = crate::gltf::load_scene(&self.device, &path, context)?;

        context.set_asset(scene);

        Ok(())
    }

    fn extensions(&self) -> &[&str] {
        &["gltf", "glb"]
    }
}
#[cfg(test)]
mod tests {
    use hikari_asset::*;
    use simple_logger::SimpleLogger;

    use crate::{texture::TextureLoader, Material, MaterialLoader, Scene, Texture2D};
    #[test]
    fn sponza() {
        use hikari_render::GfxConfig;

        use crate::GLTFLoader;

        SimpleLogger::new().init().unwrap();

        // Create a background thread which checks for deadlocks every 1s
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let deadlocks = parking_lot::deadlock::check_deadlock();
            if deadlocks.is_empty() {
                continue;
            }

            println!("{} deadlocks detected", deadlocks.len());
            for (i, threads) in deadlocks.iter().enumerate() {
                println!("Deadlock #{}", i);
                for t in threads {
                    println!("Thread Id {:#?}", t.thread_id());
                    println!("{:#?}", t.backtrace());
                }
            }
        });

        let gfx = hikari_render::Gfx::headless(GfxConfig {
            debug: true,
            features: hikari_render::Features::default(),
            vsync: true,
        })
        .unwrap();

        let mut manager = AssetManager::builder();
        manager.register_asset_type::<crate::Scene>();
        manager.register_asset_type::<crate::Texture2D>();
        manager.register_asset_type::<crate::Material>();

        manager.register_loader::<Scene, GLTFLoader>(GLTFLoader {
            device: gfx.device().clone(),
        });
        manager.register_loader::<Texture2D, TextureLoader>(TextureLoader {
            device: gfx.device().clone(),
        });
        manager.register_loader::<Material, MaterialLoader>(MaterialLoader);

        let manager = manager.build().unwrap();

        manager.load_db().expect("Failed to load Asset DB");

        let sponza: Handle<crate::Scene> = manager
            .load("../../engine_assets/models/sponza/sponza.glb", None, false)
            .expect("Failed to load sponza");

        let sponza: ErasedHandle = sponza.into();
        loop {
            manager.update::<crate::Scene>();
            manager.update::<crate::Texture2D>();
            manager.update::<crate::Material>();

            if let Some(load_status) = manager.status(&sponza) {
                match load_status {
                    LoadStatus::Loaded => break,
                    LoadStatus::Failed => panic!(),
                    _ => {}
                }
            }
        }

        manager.save_db().expect("Failed to save Asset DB");
        // For race condition in hikari_render related to Textures
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
impl Asset for Scene {
    type Settings = ();
}
