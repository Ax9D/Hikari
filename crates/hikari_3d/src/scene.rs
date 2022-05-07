use std::sync::Arc;

use hikari_asset::{Asset, Load};

use crate::{Camera, Mesh, MeshFormat};

pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub camera: Camera,
}
pub struct SceneLoader {
    pub device: Arc<hikari_render::Device>,
}
impl Load for Scene {
    type Loader = SceneLoader;

    type LoadSettings = ();

    fn load(
        loader: &Self::Loader,
        data: &[u8],
        meta: &hikari_asset::MetaData<Self>,
        context: &mut hikari_asset::LoadContext,
    ) -> Result<Self, hikari_asset::Error>
    where
        Self: Sized,
    {
        let path = &meta.data_path;
        let extension = path
            .extension()
            .ok_or(crate::Error::FailedToIdentifyFormat(
                path.as_os_str().to_owned(),
            ))?;

        let format = MeshFormat::from_extension(extension)?;

        match format {
            MeshFormat::Gltf => crate::gltf::load_scene(&loader.device, path, data, context),
            MeshFormat::Fbx => {
                todo!()
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use hikari_asset::*;
    use simple_logger::SimpleLogger;
    use winit::platform::unix::EventLoopExtUnix;

    use crate::texture::TextureLoader;
    #[test]
    fn sponza() {
        use std::sync::Arc;

        use hikari_render::GfxConfig;
        use rayon::ThreadPoolBuilder;

        use crate::SceneLoader;

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

        let eloop = winit::event_loop::EventLoop::<()>::new_any_thread();
        let window = winit::window::Window::new(&eloop).unwrap();
        let gfx = hikari_render::Gfx::new(
            &window,
            GfxConfig {
                debug: true,
                features: hikari_render::Features::default(),
                vsync: true,
            },
        )
        .unwrap();

        let thread_pool = Arc::new(ThreadPoolBuilder::new().build().unwrap());
        let mut meshes = Assets::<crate::Scene>::new();
        let mut textures = Assets::<crate::Texture2D>::new();
        let mut materials = Assets::<crate::Material>::new();
        let mut manager = AssetManagerBuilder::new(&thread_pool);
        manager.add_loader(
            SceneLoader {
                device: gfx.device().clone(),
            },
            &meshes,
        );
        manager.add_loader(
            TextureLoader {
                device: gfx.device().clone(),
            },
            &textures,
        );
        manager.add_loader((), &materials);
        let manager = manager.build();

        let sponza: Handle<crate::Scene> = manager
            .load("/home/atri/sponza/sponza.glb")
            .expect("Failed to load sponza");

        let sponza: ErasedHandle = sponza.into();
        loop {
            manager.update(&mut meshes);
            manager.update(&mut textures);
            manager.update(&mut materials);

            if let Some(load_status) = manager.get_load_status(&sponza) {
                if matches!(load_status, LoadStatus::Loading) {
                    continue;
                }
            }
        }
    }
}
impl Asset for Scene {
    const NAME: &'static str = "Mesh";

    fn extensions<'a>() -> &'a [&'static str] {
        &["gltf"]
    }
}
