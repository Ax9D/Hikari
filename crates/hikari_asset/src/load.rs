use std::{
    any::Any,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{Asset, AssetManager, IO, BufReadSeek};

pub struct LoadContext {
    asset_dir: PathBuf,
    rel_path: PathBuf,
    io: Arc<dyn IO>,
    reader: Box<dyn BufReadSeek +  Send + Sync + 'static>,
    settings: Box<dyn Any + Send + Sync + 'static>,
    asset: Option<Box<dyn Any + Send + Sync + 'static>>,
    reload: bool,
    ass_man: AssetManager,
}
impl LoadContext {
    pub fn new<T: Asset>(
        asset_dir: PathBuf,
        rel_path: PathBuf,
        io: Arc<dyn IO>,
        reader: Box<dyn BufReadSeek + Send + Sync + 'static>,
        settings: T::Settings,
        reload: bool,
        ass_man: AssetManager,
    ) -> Self {
        Self {
            asset_dir,
            rel_path,
            io,
            reader,
            settings: Box::new(settings),
            asset: None,
            reload,
            ass_man,
        }
    }
    pub fn io(&self) -> &dyn IO {
        &*self.io
    }
    /// Returns absolute path of asset directory
    pub fn asset_dir(&self) -> &Path {
        &self.asset_dir
    }
    /// Return path of asset relative to asset directory
    pub fn path(&self) -> &Path {
        &self.rel_path
    }
    pub fn reader(&mut self) -> &mut impl BufReadSeek {
        &mut self.reader
    }
    pub fn settings<T: Asset>(&self) -> &T::Settings {
        self.settings.downcast_ref().unwrap()
    }
    pub fn is_reload(&self) -> bool {
        self.reload
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

pub trait Loader: Send + Sync + 'static {
    fn extensions(&self) -> &[&str];
    fn load(&self, ctx: &mut LoadContext) -> anyhow::Result<()>;
}
