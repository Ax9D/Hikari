use std::{fs::File, path::Path};

use hikari::asset::{AssetManager, AssetStorage, Handle};
use serde::{Deserialize, Serialize};

use crate::scene::Scene;

pub const PROJECT_EXTENSION: &str = "hikari";

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    engine_version: String,
    scenes: Vec<Handle<Scene>>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            name,
            engine_version: env!("CARGO_PKG_VERSION").into(),
            scenes: vec![],
        }
    }
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        Ok(serde_yaml::from_reader(File::open(path)?)?)
    }
    pub fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        // let mut file_name = PathBuf::from(path);
        // file_name.set_extension(PROJECT_EXTENSION);

        // let path = path.as_ref().join(&file_name);

        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)?;
        Ok(serde_yaml::to_writer(file, self)?)
    }
    pub fn create_scene(
        &mut self,
        path: impl AsRef<Path>,
        scene: Scene,
        manager: &AssetManager,
        storage: &mut AssetStorage,
    ) -> anyhow::Result<Handle<Scene>> {
        // let mut name_with_ext = PathBuf::from(name);
        // name_with_ext.set_extension(SCENE_EXTENSION);
        // let full_path = path.as_ref().join(name_with_ext);

        let mut scenes = storage.get_mut::<Scene>().unwrap();
        let handle = manager.create(path.as_ref(), scene, &mut scenes)?;
        manager.save(&handle, scenes)?;
        self.scenes.push(handle.clone());
        Ok(handle)
    }
    pub fn delete_scene(&mut self, handle: &Handle<Scene>) {
        let mut remove_ix = None;
        for (ix, scene) in self.scenes.iter().enumerate() {
            if scene == handle {
                remove_ix = Some(ix);
            }
        }

        remove_ix.map(|ix| self.scenes.remove(ix));
    }

    pub fn scenes(&self) -> &[Handle<Scene>] {
        &self.scenes
    }
}

#[test]
fn serialize() {
    println!(
        "{}",
        serde_yaml::to_string(&Project::new("Test".into())).unwrap()
    );
}
