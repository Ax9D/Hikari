use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{path, Asset};

pub const METADATA_EXTENSION: &str = "meta";

#[derive(Serialize, Deserialize)]
pub(crate) struct MetaData<T: Asset> {
    pub source: PathBuf,
    pub is_standalone: bool,
    pub uuid: Uuid,
    pub settings: T::Settings,
}

impl<T: Asset> MetaData<T> {
    pub fn for_file(source: &Path) -> Option<MetaData<T>> {
        let meta_path = path::add_extension(&source, METADATA_EXTENSION);

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
    pub fn save(&self, save_path: &Path) -> anyhow::Result<()> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path::add_extension(&save_path, METADATA_EXTENSION))?;
        serde_yaml::to_writer(file, self)?;

        Ok(())
    }
}
