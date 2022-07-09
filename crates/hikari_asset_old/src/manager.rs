use std::{
    any::Any,
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::assets::{Assets, HandleAllocator};
use crate::handle::{ErasedHandle, Handle, HandleIndex, RefCounter};
use crate::{
    asset::Asset,
    meta::{AssetType, MetaData},
};
use ::serde::{Deserialize, Serialize};
use parking_lot::{Mutex, MutexGuard};
use rayon::ThreadPool;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub enum Source {
    Path(PathBuf),
    RawData(PathBuf, Vec<u8>),
}
impl Source {
    pub fn path_buf(&self) -> &PathBuf {
        match self {
            Source::Path(path_buf) | Source::RawData(path_buf, _) => path_buf,
        }
    }
}

enum LoadStrategy {
    FromPath,
    FromData(Vec<u8>),
}
pub struct LoadContext {
    path: PathBuf,
    dependencies: Vec<PathBuf>,
    manager: AssetManager,
}
impl LoadContext {
    pub fn load_dependency<T: Asset>(
        &mut self,
        dependency: Source,
        settings: T::LoadSettings,
    ) -> Result<Handle<T>, anyhow::Error> {
        //self.manager.load_subasset::<T>(data, todo!(), todo!());
        let dep_path = dependency.path_buf().clone();
        self.dependencies.push(dep_path);

        self.manager
            .load_dependency::<T>(&self.path, dependency, settings)
    }
}
#[derive(Clone, Copy, Debug)]
pub enum LoadStatus {
    Loading,
    Loaded,
    Unloaded,
}

struct LoadResult<T> {
    handle: Handle<T>,
    dependencies: Vec<PathBuf>,
    result: Result<T, anyhow::Error>,
}
struct AssetLoader<T: Asset> {
    handle_allocator: Arc<HandleAllocator>,
    asset_loader: Arc<T::Loader>,

    progress_recv: flume::Receiver<LoadResult<T>>,
    progress_send: flume::Sender<LoadResult<T>>,
    maps: Mutex<MetaMaps<T>>,
    ref_counter: Mutex<RefCounter>,
}

impl<T: Asset> AssetLoader<T> {
    pub fn new(asset_loader: T::Loader, assets: &Assets<T>) -> Self {
        let (progress_send, progress_recv) = flume::unbounded();
        let handle_allocator = assets.handle_allocator().clone();
        let ref_counter = Mutex::new(RefCounter::default());
        Self {
            handle_allocator,
            asset_loader: Arc::new(asset_loader),
            progress_recv,
            progress_send,
            ref_counter,
            maps: Mutex::new(MetaMaps::new()),
        }
    }
    #[must_use]
    pub fn load_dependency(
        &self,
        parent_path: &Path,
        source: Source,
        settings: T::LoadSettings,
        ass_man: AssetManager,
    ) -> Result<Handle<T>, anyhow::Error> {
        let path = source.path_buf();
        println!("Trying to load dependency {:#?}", path);
        let mut maps = self.maps.lock();
        let mut meta = MetaData::from_path_or_with(&path, || {
            maps.path_to_meta_data(&path).cloned().unwrap_or(MetaData {
                uuid: Uuid::new_v4(),
                data_path: path.to_owned(),
                asset_type: AssetType::Dependent,
                dependencies: Vec::new(),
                settings: T::LoadSettings::default(),
            })
        })?;

        meta.settings = settings;

        assert!(
            meta.asset_type == AssetType::Dependent,
            "{:?}",
            meta.data_path
        );
        let alloc_handle = if let Some(existing_handle) = maps.uuid_to_handle(&meta.uuid) {
            assert!(existing_handle.is_weak());
            println!(
                "Existing handle {} exists... using that",
                existing_handle.index()
            );
            existing_handle.clone_strong()
        } else {
            let handle = self.alloc_handle();
            println!("Allocating handle {}", handle.index());
            maps.register_handle(handle.clone_weak(), meta.clone());
            handle
        };

        {
            let mut load_statuses = ass_man.inner.load_statuses.lock();
            let weak_handle: ErasedHandle = alloc_handle.clone_weak().into();
            // let parent_handle = maps.path_to_handle(parent_path).unwrap().clone().into();
            // load_statuses.set_dependency_loading(parent_handle, weak_handle.clone());
            load_statuses.set_status(weak_handle, LoadStatus::Loading);
        }
        let strat = match source {
            Source::Path(_) => LoadStrategy::FromPath,
            Source::RawData(_, data) => LoadStrategy::FromData(data),
        };

        self.load_with_handle(&alloc_handle, strat, &meta, &ass_man);

        assert!(!alloc_handle.is_weak());
        return Ok(alloc_handle);
    }
    #[must_use]
    fn load(
        &self,
        path: &Path,
        settings: Option<T::LoadSettings>,
        ass_man: &AssetManager,
        force: bool,
    ) -> Result<Handle<T>, anyhow::Error> {
        let mut maps = self.maps.lock();

        let mut meta = MetaData::from_path_or_with(&path, || {
            maps.path_to_meta_data(&path).cloned().unwrap_or(MetaData {
                uuid: Uuid::new_v4(),
                data_path: path.to_owned(),
                asset_type: AssetType::Standalone,
                dependencies: Vec::new(),
                settings: T::LoadSettings::default(),
            })
        })?;

        println!("Trying to load {:#?}", path);
        //Override settings if provided
        if let Some(settings) = settings {
            meta.settings = settings;
        }

        if let Some(existing_handle) = maps.uuid_to_handle(&meta.uuid) {
            assert!(existing_handle.is_weak());
            let erased = existing_handle.clone().into();
            println!(
                "Existing handle {} exists... using that",
                existing_handle.index()
            );
            //If already loaded/loading and a force load is not requested
            if !force
                && matches!(
                    ass_man.inner.load_statuses.lock().get_status(&erased),
                    Some(LoadStatus::Loaded | LoadStatus::Loading)
                )
            {
                return Ok(existing_handle.clone_strong());
            } else {
                // If the current asset is a dependency, skip loading entirely, dependencies are loaded by their parent standalone assets
                if meta.is_standalone_asset() {
                    self.load_with_handle(existing_handle, LoadStrategy::FromPath, &meta, ass_man);
                }
                return Ok(existing_handle.clone_strong());
            }
        } else {
            let alloc_handle = self.alloc_handle();
            println!("Allocating handle {}", alloc_handle.index());
            maps.register_handle(alloc_handle.clone_weak(), meta.clone());
            ass_man
                .inner
                .load_statuses
                .lock()
                .set_status(alloc_handle.clone_weak().into(), LoadStatus::Loading);

            self.load_with_handle(&alloc_handle, LoadStrategy::FromPath, &meta, ass_man);

            return Ok(alloc_handle);
        }
    }
    fn load_with_handle(
        &self,
        handle: &Handle<T>,
        strat: LoadStrategy,
        meta: &MetaData<T>,
        ass_man: &AssetManager,
    ) {
        assert!(!handle.is_weak());

        fn load_task<T: Asset>(
            loader: Arc<T::Loader>,
            strat: LoadStrategy,
            meta: &MetaData<T>,
            context: &mut LoadContext,
        ) -> Result<T, anyhow::Error> {
            let data = match strat {
                LoadStrategy::FromPath => std::fs::read(&context.path)?,
                LoadStrategy::FromData(data) => data,
            };
            let asset = T::load(&loader, &data, meta, context)?;
            Ok(asset)
        }

        let ass_man_cl = ass_man.clone();
        let loader = self.asset_loader.clone();
        let path = meta.data_path.clone();
        let sender = self.progress_send.clone();
        let handle = handle.clone();
        let meta = meta.clone();
        ass_man.inner.threadpool.spawn(move || {
            let mut context = LoadContext {
                path,
                manager: ass_man_cl,
                dependencies: Vec::new(),
            };

            let result = load_task::<T>(loader, strat, &meta, &mut context);
            sender
                .send(LoadResult {
                    handle,
                    result,
                    dependencies: context.dependencies,
                })
                .expect("Failed to send load result");
        });
    }
    fn update_refcounts(&self, ass_man: &AssetManager, assets: &mut Assets<T>) {
        let mut ref_counter = self.ref_counter.lock();

        for op in self.handle_allocator.ref_op_channel().try_iter() {
            ref_counter.process_op(op);
        }

        let mut load_statuses = ass_man.inner.load_statuses.lock();
        ref_counter.remove_with(|index| {
            let load_status = load_statuses
                .get_status(&ErasedHandle::use_for_hashing::<T>(index))
                .expect("Load status not updated properly");

            assets.remove(index);
            self.dealloc_handle(index);
            let meta = ass_man.meta_maps::<T>().unwrap();
            let meta = meta
                .handle_to_metadata(
                    &ErasedHandle::use_for_hashing::<T>(index)
                        .into_typed()
                        .unwrap(),
                )
                .unwrap();
            log::error!("removing handle with path {} {:#?}", index, meta.data_path);
            *load_status = LoadStatus::Unloaded;
        });
    }
    fn update(&self, ass_man: &AssetManager, assets: &mut Assets<T>) {
        for progress in self.progress_recv.try_iter() {
            let mut maps = self.maps.lock();
            match progress.result {
                Ok(asset) => {
                    let meta = maps
                        .handle_to_metadata_mut(&progress.handle)
                        .expect("Handle was never registered");
                    assets.insert(progress.handle.index(), asset);

                    // Update dependencies
                    if &meta.dependencies != &progress.dependencies {
                        meta.dependencies = progress.dependencies;
                    }

                    {
                        let mut load_statuses = ass_man.inner.load_statuses.lock();
                        let erased_handle = progress.handle.into();

                        // load_statuses.set_dependency_loaded(&erased_handle);
                        let load_status = load_statuses
                            .get_status(&erased_handle)
                            .expect("Load status not updated properly");
                        *load_status = LoadStatus::Loaded;
                    }

                    log::info!("Loaded {:#?}", meta.data_path);
                    meta.save().expect("Failed to save metadata to disk");
                }
                Err(err) => {
                    let meta = maps
                        .deregister_handle(&progress.handle)
                        .expect("Handle was never registered");
                    ass_man
                        .inner
                        .load_statuses
                        .lock()
                        .remove(&progress.handle.into());
                    log::error!("Failed to load asset: {:?}, Error: {}", meta.data_path, err);
                }
            }
        }
        self.update_refcounts(ass_man, assets);
    }
    fn alloc_handle(&self) -> Handle<T> {
        self.handle_allocator.allocate::<T>()
    }
    fn dealloc_handle(&self, handle_index: HandleIndex) {
        self.handle_allocator.deallocate(handle_index);
    }
    fn get_uuid(&self, handle: &Handle<T>) -> Option<Uuid> {
        let maps = self.maps.lock();
        Some(maps.handle_to_metadata(handle)?.uuid)
    }
    fn meta_maps(&self) -> MutexGuard<MetaMaps<T>> {
        self.maps.lock()
    }
}
pub struct MetaMaps<T: Asset> {
    uuid_to_handle: HashMap<Uuid, Handle<T>>,
    handle_to_metadata: HashMap<Handle<T>, MetaData<T>>,
    path_to_handle: HashMap<PathBuf, Handle<T>>,
}

impl<T: Asset> MetaMaps<T> {
    pub fn new() -> Self {
        MetaMaps {
            uuid_to_handle: Default::default(),
            handle_to_metadata: Default::default(),
            path_to_handle: Default::default(),
        }
    }
    pub fn uuid_to_handle(&self, uuid: &Uuid) -> Option<&Handle<T>> {
        self.uuid_to_handle.get(uuid)
    }
    pub fn path_to_handle(&self, path: &Path) -> Option<&Handle<T>> {
        self.path_to_handle.get(path)
    }
    pub fn path_to_meta_data(&self, path: &Path) -> Option<&MetaData<T>> {
        let handle = self.path_to_handle(path)?;
        self.handle_to_metadata(handle)
    }
    pub fn handle_to_uuid(&self, handle: &Handle<T>) -> Option<&Uuid> {
        self.handle_to_metadata.get(handle).map(|meta| &meta.uuid)
    }
    pub fn handle_to_metadata(&self, handle: &Handle<T>) -> Option<&MetaData<T>> {
        self.handle_to_metadata.get(handle)
    }
    pub fn handle_to_metadata_mut(&mut self, handle: &Handle<T>) -> Option<&mut MetaData<T>> {
        self.handle_to_metadata.get_mut(handle)
    }
    pub(crate) fn register_handle(&mut self, handle: Handle<T>, meta: MetaData<T>) {
        assert!(handle.is_weak());

        self.path_to_handle
            .insert(meta.data_path.clone(), handle.clone());
        self.uuid_to_handle
            .insert(meta.uuid.clone(), handle.clone());
        self.handle_to_metadata.insert(handle, meta);
    }
    pub(crate) fn deregister_handle(&mut self, handle: &Handle<T>) -> Option<MetaData<T>> {
        let meta = self.handle_to_metadata.remove(handle)?;
        log::debug!("Deregistering handle with path {:?}", meta.data_path);
        self.uuid_to_handle.remove(&meta.uuid);
        self.path_to_handle.remove(&meta.data_path);
        Some(meta)
    }
}
struct LoadStatuses {
    inner: HashMap<ErasedHandle, LoadStatus>,
    // dep_count: HashMap<ErasedHandle, u32>, // No. of dependencies that are still loading
    // dep_to_parent: HashMap<ErasedHandle, ErasedHandle>
}
impl LoadStatuses {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
            // dep_count: Default::default(),
            // dep_to_parent: Default::default()
        }
    }
    fn set_status(&mut self, handle: ErasedHandle, status: LoadStatus) {
        assert!(handle.is_weak());
        self.inner.insert(handle.clone(), status);
        //self.dep_count.insert(handle, 0);
    }
    fn get_status(&mut self, handle: &ErasedHandle) -> Option<&mut LoadStatus> {
        self.inner.get_mut(handle)
    }
    // fn set_dependency_loading(&mut self, parent: ErasedHandle, dependency: ErasedHandle) {
    //     self.inc_count(&parent);
    //     //self.dep_to_parent.insert(dependency, parent);
    // }
    // fn set_dependency_loaded(&mut self, dependency: &ErasedHandle) {
    //     let parent = self.dep_to_parent.remove(&dependency).unwrap();
    //     if self.dec_count(&parent) == 0 {
    //         self.set_status(parent, LoadStatus::Loaded);
    //     }
    // }

    fn remove(&mut self, handle: &ErasedHandle) -> Option<LoadStatus> {
        //let count = self.dep_count.remove(handle)?;
        self.inner.remove(handle)
    }
    // fn get_dep_count(&mut self, handle: &ErasedHandle) -> Option<&mut u32> {
    //     self.dep_count.get_mut(handle)
    // }
    // fn dec_count(&mut self, handle: &ErasedHandle) -> u32 {
    //     let dec_count = self.get_dep_count(handle).unwrap();
    //     *dec_count-=1;

    //     *dec_count
    // }
    // fn inc_count(&mut self, handle: &ErasedHandle) -> u32 {
    //     let dec_count = self.get_dep_count(handle).unwrap();
    //     *dec_count+=1;

    //     *dec_count
    // }
}
struct AssetManagerInner {
    loaders: HashMap<&'static str, Box<dyn Any + Send + Sync + 'static>>,
    extension_to_asset_name: HashMap<OsString, &'static str>,
    load_statuses: Mutex<LoadStatuses>,
    threadpool: Arc<ThreadPool>,
}

impl AssetManagerInner {
    fn new(threadpool: &Arc<ThreadPool>) -> Self {
        Self {
            loaders: HashMap::new(),
            threadpool: threadpool.clone(),
            extension_to_asset_name: HashMap::new(),
            load_statuses: Mutex::new(LoadStatuses::new()),
        }
    }
    fn get_loader<T: Asset>(&self) -> Result<&AssetLoader<T>, anyhow::Error> {
        Ok(self
            .loaders
            .get(T::NAME)
            .ok_or(anyhow::anyhow!(
                "Failed to get loader for asset {}",
                T::NAME
            ))?
            .downcast_ref::<AssetLoader<T>>()
            .expect("Asset Loader is not of expected type"))
    }
    fn add_loader<T: Asset>(&mut self, loader: T::Loader, assets: &Assets<T>) {
        self.register_extensions(T::extensions(), T::NAME);

        let loader = AssetLoader::<T>::new(loader, assets);
        if self.loaders.insert(T::NAME, Box::new(loader)).is_some() {
            panic!(
                "Cannot register loader twice! Loader for {} already exists",
                T::NAME
            );
        }
    }
    fn register_extensions(&mut self, extensions: &[&'static str], asset_name: &'static str) {
        for extension in extensions {
            let previous = self
                .extension_to_asset_name
                .insert(OsString::from(extension), asset_name);
            if let Some(previous) = previous {
                panic!(
                    "Extension {} has already been associated with loader of asset {}",
                    extension, previous
                );
            }
        }
    }
    fn get_uuid<T: Asset>(&self, handle: &Handle<T>) -> Option<Uuid> {
        self.get_loader::<T>().ok()?.get_uuid(handle)
    }
    fn meta_maps<T: Asset>(&self) -> Result<MutexGuard<MetaMaps<T>>, anyhow::Error> {
        Ok(self.get_loader::<T>()?.meta_maps())
    }
    fn get_load_status(&self, handle: &ErasedHandle) -> Option<LoadStatus> {
        self.load_statuses.lock().get_status(handle).cloned()
    }
}
#[derive(Clone)]
pub struct AssetManager {
    inner: Arc<AssetManagerInner>,
}
impl AssetManager {
    pub fn load<T: Asset>(&self, path: impl AsRef<Path>) -> Result<Handle<T>, anyhow::Error> {
        let loader = self.inner.get_loader::<T>()?;
        let handle = loader.load(path.as_ref(), None, self, false)?;
        Ok(handle)
    }
    pub(crate) fn load_dependency<T: Asset>(
        &self,
        parent_path: &Path,
        dependency: Source,
        settings: T::LoadSettings,
    ) -> Result<Handle<T>, anyhow::Error> {
        let loader = self.inner.get_loader()?;
        loader.load_dependency(parent_path, dependency, settings, self.clone())
    }
    pub fn load_with_settings<T: Asset>(
        &self,
        path: impl AsRef<Path>,
        settings: T::LoadSettings,
        reload: bool,
    ) -> Result<Handle<T>, anyhow::Error> {
        let loader = self.inner.get_loader::<T>()?;
        let handle = loader.load(path.as_ref(), Some(settings), self, reload)?;
        Ok(handle)
    }
    pub fn update<T: Asset>(&self, assets: &mut Assets<T>) {
        let loader = self.inner.get_loader::<T>().unwrap();
        loader.update(self, assets);
    }
    pub fn get_uuid<T: Asset>(&self, handle: &Handle<T>) -> Option<Uuid> {
        self.inner.get_uuid(handle)
    }
    pub fn meta_maps<T: Asset>(&self) -> Result<MutexGuard<MetaMaps<T>>, anyhow::Error> {
        self.inner.meta_maps()
    }
    pub fn get_load_status(&self, handle: &ErasedHandle) -> Option<LoadStatus> {
        self.inner.get_load_status(handle)
    }
}
pub struct AssetManagerBuilder {
    inner: AssetManagerInner,
}
impl AssetManagerBuilder {
    pub fn new(threadpool: &Arc<ThreadPool>) -> Self {
        Self {
            inner: AssetManagerInner::new(threadpool),
        }
    }
    pub fn add_loader<T: Asset>(&mut self, loader: T::Loader, assets: &Assets<T>) {
        self.inner.add_loader(loader, assets);
    }
    pub fn build(self) -> AssetManager {
        let ass_man = AssetManager {
            inner: Arc::new(self.inner),
        };

        crate::serde::init_serde(ass_man.clone());
        ass_man
    }
}
