use std::sync::Arc;

use hikari_asset::{Asset, LoadContext, Loader};

use crate::{Camera, Mesh, MeshFormat};

pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub camera: Camera,
}
pub struct GLTFLoader {
    pub device: Arc<hikari_render::Device>,
}
impl Loader for GLTFLoader {
    fn load(&self, context: &mut LoadContext) -> anyhow::Result<()> {
        let path = match context.source() {
            hikari_asset::Source::FileSystem(path) => path.as_path(),
            hikari_asset::Source::Data(_, _) => todo!(),
        };

        let path = path.to_owned();

        let data = std::fs::read(&path)?;
        let scene = crate::gltf::load_scene(&self.device, &path, &data, context)?;

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
        use std::sync::Arc;

        use hikari_render::GfxConfig;
        use rayon::ThreadPoolBuilder;

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

        let thread_pool = Arc::new(ThreadPoolBuilder::new().build().unwrap());
        let mut meshes = AssetPool::<crate::Scene>::default();
        let mut textures = AssetPool::<crate::Texture2D>::default();
        let mut materials = AssetPool::<crate::Material>::default();
        let mut manager = AssetManager::with_threadpool(thread_pool);
        manager.register_asset(&meshes);
        manager.register_asset(&textures);
        manager.register_asset(&materials);
        manager.add_loader::<Scene, GLTFLoader>(GLTFLoader {
            device: gfx.device().clone(),
        });
        manager.add_loader::<Texture2D, TextureLoader>(TextureLoader {
            device: gfx.device().clone(),
        });
        manager.add_loader::<Material, MaterialLoader>(MaterialLoader);

        hikari_asset::serde::init(manager.clone());

        let sponza: Handle<crate::Scene> = manager
            .load("/home/atri/sponza/sponza.glb".as_ref())
            .expect("Failed to load sponza");

        let sponza: ErasedHandle = sponza.into();
        loop {
            manager.update(&mut meshes).expect("Failed meshes");
            manager.update(&mut textures).expect("Failed textures");
            manager.update(&mut materials).expect("Failed materials");

            if let Some(load_status) = manager.load_status(&sponza) {
                match load_status {
                    LoadStatus::Loaded => break,
                    LoadStatus::Failed => panic!(),
                    _=> {}
                }
            }
        }

        // For race condition in hikari_render related to Textures
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
impl Asset for Scene {
    type Settings = ();
}
