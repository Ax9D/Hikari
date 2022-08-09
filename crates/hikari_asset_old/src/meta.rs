use std::{
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Load;

const META_EXT: &str = "hmeta";

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum AssetType {
    Standalone,
    Dependent,
}

// impl Serialize for AssetType {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer {
//         serializer.serialize
//     }
// }

// impl<'de> Deserialize<'de> for AssetType {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de> {
//         todo!()
//     }
// }

#[derive(Serialize, Deserialize)]
pub struct MetaData<S: Load> {
    pub uuid: Uuid,
    pub data_path: PathBuf,
    pub asset_type: AssetType,
    pub dependencies: Vec<PathBuf>,
    pub settings: S::LoadSettings,
}

impl<S: Load> Clone for MetaData<S> {
    fn clone(&self) -> Self {
        Self {
            data_path: self.data_path.clone(),
            uuid: self.uuid.clone(),
            asset_type: self.asset_type.clone(),
            dependencies: self.dependencies.clone(),
            settings: self.settings.clone(),
        }
    }
}
impl<S: Load> MetaData<S> {
    fn validate(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    ///Path corresponds to the data and not the metadata
    pub(crate) fn from_path_or_with(
        data_path: impl AsRef<Path>,
        dep: impl FnOnce() -> MetaData<S>,
    ) -> Result<Self, anyhow::Error> {
        let data_path = data_path.as_ref();
        /*
            If exists read from it
            else
            create default and read from it
        */
        let mut meta_path = data_path.to_owned();
        add_extension(&mut meta_path, META_EXT);

        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&meta_path)?;

        let meta = serde_yaml::from_reader(file).unwrap_or_else(|_| (dep)());

        meta.validate()?;
        Ok(meta)
    }
    pub(crate) fn save(&self) -> Result<(), anyhow::Error> {
        let mut meta_path = self.data_path.clone();
        add_extension(&mut meta_path, META_EXT);

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(meta_path)?;

        file.write_all(serde_yaml::to_string::<Self>(self)?.as_bytes())?;

        Ok(())
    }
    pub fn is_standalone_asset(&self) -> bool {
        matches!(self.asset_type, AssetType::Standalone)
    }
}

fn add_extension(path: &mut std::path::PathBuf, extension: impl AsRef<std::path::Path>) {
    match path.extension() {
        Some(ext) => {
            let mut ext = ext.to_os_string();
            ext.push(".");
            ext.push(extension.as_ref());
            path.set_extension(ext)
        }
        None => path.set_extension(extension.as_ref()),
    };
}
