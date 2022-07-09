use std::{
    any::Any,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Asset, AssetManager};
pub struct LoadContext {
    source: Source,
    settings: Box<dyn Any + Send + Sync + 'static>,
    asset: Option<Box<dyn Any + Send + Sync + 'static>>,
    ass_man: AssetManager,
}
impl LoadContext {
    pub fn new<T: Asset>(source: Source, settings: T::Settings, ass_man: AssetManager) -> Self {
        Self {
            source,
            settings: Box::new(settings),
            asset: None,
            ass_man,
        }
    }
    pub fn source(&self) -> &Source {
        &self.source
    }
    pub fn settings<T: Asset>(&self) -> &T::Settings {
        self.settings.downcast_ref().unwrap()
    }
    pub fn asset_manager(&self) -> &AssetManager {
        &self.ass_man
    }
    pub fn set_asset<T: Asset>(&mut self, asset: T) {
        assert!(self.asset.is_none());

        self.asset = Some(Box::new(asset));
    }
    pub(crate) fn take_asset<T: Asset>(&mut self) -> Option<T> {
        self.asset
            .take()
            .map(|any_asset| *any_asset.downcast::<T>().unwrap())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Source {
    FileSystem(PathBuf),
    Data(PathBuf, Vec<u8>),
}

impl Source {
    pub fn path(&self) -> PathBuf {
        let cwd = std::env::current_dir().unwrap();
        match self {
            Source::FileSystem(path) | Source::Data(path, _) => path.clone(),
        }
    }
    pub fn is_filesystem(&self) -> bool {
        matches!(self, Self::FileSystem(_))
    }
}

pub trait Loader: Send + Sync + 'static {
    fn extensions(&self) -> &[&str];
    fn load(&self, ctx: &mut LoadContext) -> anyhow::Result<()>;
}
