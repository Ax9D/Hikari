use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Asset;

pub const METADATA_EXTENSION: &str = "meta";

#[derive(Serialize, Deserialize)]
pub(crate) struct MetaData<T: Asset> {
    pub source_path: PathBuf,
    pub is_standalone: bool,
    pub uuid: Uuid,
    pub settings: T::Settings,
}

impl<T: Asset> MetaData<T> {
    fn add_extension(path: &Path, extension: impl AsRef<std::path::Path>) -> PathBuf {
        let mut path = path.to_owned();
        match path.extension() {
            Some(ext) => {
                let mut ext = ext.to_os_string();
                ext.push(".");
                ext.push(extension.as_ref());
                path.set_extension(ext)
            }
            None => path.set_extension(extension.as_ref()),
        };

        path
    }
    pub fn for_file(source: &Path) -> Option<MetaData<T>> {
        let meta_path = Self::add_extension(&source, METADATA_EXTENSION);

        if meta_path.exists() {
            let file_text = std::fs::read_to_string(meta_path).ok()?;
            let meta = serde_yaml::from_str::<Self>(&file_text);
            match meta {
                Ok(meta) => {
                    return Some(meta);
                }
                Err(_) => {}
            };
        }

        None
    }
    pub fn save(&self) -> anyhow::Result<()> {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(Self::add_extension(&self.source_path, METADATA_EXTENSION))?;
        serde_yaml::to_writer(file, self)?;

        Ok(())
    }
}
