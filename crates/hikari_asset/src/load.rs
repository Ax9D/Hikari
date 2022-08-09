use std::{
    any::Any,
    path::{PathBuf, Path}, sync::Arc, io::{Read},
};


use crate::{meta::MetaData, AssetManager, Asset, Handle, HandleAllocator};

#[derive(Debug, Clone)]
pub enum Source {
    FileSystem(PathBuf),
    Data(PathBuf, Vec<u8>),
}

impl Source {
    pub fn absolute_path(&self, asset_dir: &Path) -> PathBuf {
        asset_dir.join(self.relative_path())
    }
    pub fn relative_path(&self) -> &Path {
        match self {
            Source::FileSystem(path) | Source::Data(path, _) => { 
                assert!(path.is_relative());    
                path 
            },
        }
    }
    pub fn is_filesystem(&self) -> bool {
        matches!(self, Self::FileSystem(_))
    }
}

pub(crate) struct LoadResult<T: Asset> {
    pub asset: Result<T, anyhow::Error>,
    pub handle: Handle<T>,
    pub meta: MetaData<T>,
}

pub(crate) struct LoadState<T: Asset> {
    pub handle_allocator: Arc<HandleAllocator>,
    pub load_send: flume::Sender<LoadResult<T>>,
    pub load_recv: flume::Receiver<LoadResult<T>>,
}

impl<T: Asset> LoadState<T> {
    pub fn new(handle_allocator: Arc<HandleAllocator>) -> Self {
        let (load_send, load_recv) = flume::unbounded();
        Self {
            handle_allocator,
            load_send,
            load_recv,
        }
    }
}

pub trait Loader: Send + Sync + 'static {
    fn extensions(&self) -> &[&str];
    fn load(&self, ctx: &mut LoadContext) -> anyhow::Result<()>;
}
pub struct LoadContext {
    abs_path: PathBuf,
    reader: Box<dyn Read + Send + Sync + 'static>,
    settings: Box<dyn Any + Send + Sync + 'static>,
    asset: Option<Box<dyn Any + Send + Sync + 'static>>,
    ass_man: AssetManager,
}
impl LoadContext {
    pub fn new<T: Asset>(abs_path: PathBuf, reader: Box<dyn Read + Send + Sync + 'static>, settings: T::Settings, ass_man: AssetManager) -> Self {
        Self {
            abs_path,
            reader,
            settings: Box::new(settings),
            asset: None,
            ass_man,
        }
    }
    pub fn path(&self) -> &Path {
        &self.abs_path
    }
    pub fn reader(&mut self) -> &mut impl Read {
        &mut self.reader
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