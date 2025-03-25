use std::{
    fs::File,
    path::{Path, PathBuf},
};

use hikari::{asset::{AssetManager, Handle}, core::World};
use serde::{Deserialize, Serialize};

pub const PROJECT_EXTENSION: &str = "hikari";

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    engine_version: String,
    worlds: Vec<PathBuf>,
}

impl Project {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            engine_version: env!("CARGO_PKG_VERSION").into(),
            worlds: vec![],
        }
    }
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        Ok(serde_yaml::from_reader(File::open(path)?)?)
    }
    pub fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let mut file_name = PathBuf::from(&self.name);
        file_name.set_extension(PROJECT_EXTENSION);

        let path = path.as_ref().join(&file_name);

        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        Ok(serde_yaml::to_writer(file, self)?)
    }
    pub fn add_world(
        &mut self,
        path: impl AsRef<Path>,
        world: World,
        manager: &AssetManager,
    ) -> anyhow::Result<(Handle<World>, usize)> {
        // let mut name_with_ext = PathBuf::from(name);
        // name_with_ext.set_extension(SCENE_EXTENSION);
        // let full_path = path.as_ref().join(name_with_ext);
        let ix = self.worlds.len();
        self.worlds.push(path.as_ref().to_owned());

        let handle = manager.create(path.as_ref(), world, None)?;

        Ok((handle, ix))
    }
    pub fn delete_world(&mut self, handle: &PathBuf) {
        let mut remove_ix = None;
        for (ix, world) in self.worlds.iter().enumerate() {
            if world == handle {
                remove_ix = Some(ix);
            }
        }

        remove_ix.map(|ix| self.worlds.remove(ix));
    }

    pub fn worlds(&self) -> &[PathBuf] {
        &self.worlds
    }
}

// #[derive(Serialize, Deserialize)]
// struct SerializedProject {
//     name: String,
//     engine_version: String,
//     scenes: Vec<PathBuf>,
// }

// impl Into<Project> for SerializedProject {
//     fn into(self) -> Project {
//         let mut scenes = Vec::with_capacity(self.scenes.len());

//         for scene in self.scenes {
//             let handle: Handle<Scene> = scene.into();
//             scenes.push(handle.to_weak());
//         }

//         Project {
//             name: self.name,
//             engine_version: self.engine_version,
//             scenes,
//         }
//     }
// }

#[test]
fn serialize() {
    println!("{}", serde_yaml::to_string(&Project::new("Test")).unwrap());
}
