#[cfg(feature = "serialize")]
use crate::serialize::AnySerde;
use crate::{
    record::Record, AsPath, Asset, AssetDB, Dependencies, DynAssetPool, ErasedHandle, Handle, LoadContext, Loader, Mode, PhysicalIO, PoolMut, PoolRef, SaveContext, Saver, Unsaved, IO
};

use crate::status::*;

use anyhow::anyhow;
use once_cell::sync::OnceCell;
use parking_lot::{RwLock, Mutex};
use rayon::ThreadPool;
use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    ffi::OsStr,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::{Arc, Weak},
};
struct LoadResult<T: Asset> {
    result: anyhow::Result<(T, Dependencies)>,
    handle: ErasedHandle,
}
type QueueEnds<T> = (flume::Sender<LoadResult<T>>, flume::Receiver<LoadResult<T>>);
struct LoadQueue {
    queues: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}
impl LoadQueue {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
        }
    }
    pub fn register_asset_type<T: Asset>(&mut self) {
        let pair = flume::unbounded::<LoadResult<T>>();
        self.queues.insert(TypeId::of::<T>(), Box::new(pair));
    }
    pub fn send<T: Asset>(
        &self,
        result: LoadResult<T>,
    ) -> Result<(), flume::SendError<LoadResult<T>>> {
        let any = self.queues.get(&TypeId::of::<T>()).unwrap();
        any.downcast_ref::<QueueEnds<T>>().unwrap().0.send(result)
    }
    pub fn recv<T: Asset>(&self) -> flume::Drain<LoadResult<T>> {
        let any = self.queues.get(&TypeId::of::<T>()).unwrap();
        any.downcast_ref::<QueueEnds<T>>().unwrap().1.drain()
    }
}
struct AssetManagerInner {
    asset_db: RwLock<AssetDB>,
    unsaved: Mutex<Unsaved>,
    load_queue: Arc<LoadQueue>,
    loaders: HashMap<TypeId, Vec<Arc<dyn Loader>>>,
    savers: HashMap<TypeId, Vec<Arc<dyn Saver>>>,
    // load_statuses: LoadStatuses,
    thread_pool: Arc<ThreadPool>,
    //untyped_loaders: HashMap<TypeId, fn() -> ErasedHandle>,
    io: Arc<dyn IO>,
    #[cfg(feature = "serialize")]
    any_serde: AnySerde,
    asset_dir: RwLock<PathBuf>,
    asset_pools: HashMap<TypeId, DynAssetPool>,
}
impl AssetManagerInner {
    fn get_loader<T: Asset>(&self, path: &Path) -> anyhow::Result<&Arc<dyn Loader>> {
        let file_ext = path
            .extension()
            .ok_or_else(|| anyhow!("Couldn't determine file extension: {:#?}", path))?;
        let file_ext = file_ext.to_str().unwrap();
        let loaders = self
            .loaders
            .get(&TypeId::of::<T>())
            .ok_or_else(|| anyhow!("Loader for asset: {} not found", type_name::<T>()))?;

        for loader in loaders {
            for &extension in loader.extensions() {
                if extension.to_lowercase() == file_ext.to_lowercase() {
                    return Ok(loader);
                }
            }
        }

        Err(anyhow!(
            "Failed to find suitable loader for extension: {}",
            file_ext
        ))
    }
    pub fn read_assets<T: Asset>(&self) -> Option<PoolRef<T>> {
        self.asset_pools
            .get(&TypeId::of::<T>())
            .map(|dyn_pool| dyn_pool.read())
    }
    pub fn write_assets<T: Asset>(&self) -> Option<PoolMut<T>> {
        self.asset_pools
            .get(&TypeId::of::<T>())
            .map(|dyn_pool| dyn_pool.write())
    }
    pub fn asset_db(&self) -> &RwLock<AssetDB> {
        &self.asset_db
    }
    fn load_task<T: Asset>(
        asset_dir: PathBuf,
        rel_path: PathBuf,
        settings: T::Settings,
        reload: bool,
        io: Arc<dyn IO>,
        loader: Arc<dyn Loader>,
    ) -> anyhow::Result<(T, Dependencies)> {
        let abs_path = asset_dir.join(&rel_path);
        let reader = io.read_file(&abs_path, &Mode::read_only())?;
        let mut ctx = LoadContext::new::<T>(
            asset_dir,
            rel_path,
            io.clone(),
            reader,
            settings,
            reload,
            get_asset_manager().clone(),
        );
        loader.load(&mut ctx)?;

        let asset = ctx.take_asset::<T>().expect("Asset not set during loading");

        Ok((asset, ctx.dependencies))
    }
    fn trigger_load<T: Asset>(
        &self,
        handle: &ErasedHandle,
        path: &Path,
        settings: &T::Settings,
        reload: bool,
    ) -> anyhow::Result<()> {
        let loader = self.get_loader::<T>(path)?.clone();

        let io = self.io.clone();
        let asset_dir = self.asset_dir.read().clone();
        let rel_path = path.to_owned();
        let settings = settings.clone();
        let load_queue = self.load_queue.clone();
        let handle = handle.clone();
        self.thread_pool.spawn(move || {
            let result = Self::load_task::<T>(asset_dir, rel_path, settings, reload, io, loader);

            let load_result = LoadResult { result, handle };
            load_queue
                .send::<T>(load_result)
                .expect("Failed to send load result");
        });
        Ok(())
    }
    pub fn load<T: Asset>(
        &self,
        path: impl AsPath,
        settings: Option<T::Settings>,
        reload: bool,
    ) -> anyhow::Result<Handle<T>> {
        self.load_(path, settings, reload, false)
    }
    pub fn load_lazy<T: Asset>(
        &self,
        path: impl AsPath,
        settings: Option<T::Settings>,
    ) -> anyhow::Result<Handle<T>> {
        let handle = self.load_(path, settings, false, true)?;
        assert!(handle.is_weak());
        Ok(handle)
    }
    fn load_<T: Asset>(
        &self,
        path: impl AsPath,
        settings: Option<T::Settings>,
        reload: bool,
        lazy: bool,
    ) -> anyhow::Result<Handle<T>> {
        let path = &path.as_path(&self.asset_db().read())?;

        if !path.is_relative() {
            return Err(anyhow::anyhow!(
                "Absolute paths are not supported. Use a path relative to your asset directory"
            ));
        }

        let mut db = self.asset_db.write();
        match db.path_to_handle_and_record(path) {
            (None, None) => {
                log::info!("Not Loaded, Not Registered {:?}", path);
                //Not loaded not registered asset
                //By "loaded" I mean having an handle (irrespective of the fact if the asset was successfully loaded/failed etc)
                let settings = settings.unwrap_or_default();
                self.fresh_load::<T>(&mut db, path, settings, reload, lazy)
            }
            (None, Some(record)) => {
                log::info!("Not Loaded, Registered {:?}", path);
                //Not loaded but registered asset
                //By "loaded" I mean having an handle (irrespective of the fact if the asset was successfully loaded/failed etc)
                let settings = settings.unwrap_or_else(|| record.settings::<T>().clone());
                self.fresh_load::<T>(&mut db, path, settings, reload, lazy)
            }
            (Some((handle, load_status)), Some(record)) => {
                log::info!("Loaded, Registered {:?}", path);
                assert!(handle.is_internal());

                if lazy {
                    Ok(handle.upgrade_weak().into_typed::<T>().unwrap())

                } else {
                    let handle = handle.upgrade_strong_anyway();
                    let handle_typed = handle.clone_typed::<T>().unwrap();

                    self.registered_asset_load::<T>(&handle, record, load_status, settings, reload, lazy)?;

                    Ok(handle_typed)
                }
            }
            _ => unreachable!(),
        }

        //if let (handle, record) = db.path_to_handle_and_record(path) {
        //    self.existing_handle_load::<T>(handle, record, settings, reload, lazy)?;
        //    return Ok(handle.clone_strong().clone_typed::<T>().unwrap());
        //}
        //self.fresh_load::<T>(db, path, settings, reload, lazy)
    }
    // pub fn request_load<T: Asset>(
    //     &self,
    //     handle: &Handle<T>,
    //     settings: Option<T::Settings>,
    //     reload: bool,
    // ) -> anyhow::Result<()> {
    //     let mut db = self.asset_db().write();
    //     let erased_handle = &handle.clone_erased_as_internal();
    //     let record = db.handle_to_record_mut(erased_handle).unwrap();

    //     self.registered_asset_load::<T>(erased_handle, record, settings, reload, false)
    // }
    fn registered_asset_load<T: Asset>(
        &self,
        handle: &ErasedHandle,
        record: &mut Record,
        load_status: &mut LoadStatus,
        settings: Option<T::Settings>,
        reload: bool,
        lazy: bool,
    ) -> anyhow::Result<()> {
        if let Some(settings) = settings {
            *record.settings_mut::<T>() = settings;
        }
        let path = &record.path;

        let settings = record.settings::<T>();
        match *load_status {
            _ if lazy => {}
            LoadStatus::Unloaded | LoadStatus::Failed => {
                *load_status = LoadStatus::Loading;
                self.trigger_load::<T>(handle, path, &settings, reload)?;
            }
            _ if reload => {
                *load_status = LoadStatus::Loading;
                self.trigger_load::<T>(handle, path, &settings, reload)?;
            }
            _ => {}
        }

        Ok(())
    }
    fn fresh_load<T: Asset>(
        &self,
        db: &mut AssetDB,
        path: &Path,
        settings: T::Settings,
        reload: bool,
        lazy: bool,
    ) -> anyhow::Result<Handle<T>> {
        let handle = {
            let asset_pool = self.read_assets::<T>().expect("Asset type not registered");
            asset_pool.handle_allocator().allocate::<T>()
        };

        let erased_handle = handle.clone_erased_as_internal();

        // if lazy {
        //     self.load_statuses
        //         .insert(&erased_handle.into(), LoadStatus::Unloaded);
        // } else {
        //     self.load_statuses
        //         .insert(&erased_handle.into(), LoadStatus::Loading);
        // }
        
        if lazy {
            db.create_or_update_record::<T>(&erased_handle, path, settings.clone(), LoadStatus::Unloaded);
            Ok(handle.to_weak())
        } else {
            db.create_or_update_record::<T>(&erased_handle, path, settings.clone(), LoadStatus::Loading);
            self.trigger_load::<T>(&erased_handle, path, &settings, reload)?;

            assert!(handle.strong_count() == 1);

            Ok(handle)
        }
    }
    pub fn wait_for_load<T: Asset>(&self, handle: &Handle<T>) -> LoadStatus {
        let erased = handle.clone_erased();
        loop {
            self.update::<T>();

            let Some(status) = self.status(&erased) else { continue };

            if status == LoadStatus::Loaded || status == LoadStatus::Failed {
                return status;
            }

        }
    }
    fn get_saver<T: Asset>(&self, extension: &OsStr) -> anyhow::Result<&Arc<dyn Saver>> {
        let file_ext = extension.to_str().unwrap();
        let savers = self
            .savers
            .get(&TypeId::of::<T>())
            .ok_or_else(|| anyhow!("Saver for asset: {} not found", type_name::<T>()))?;

        for loader in savers {
            for &extension in loader.extensions() {
                if extension.to_lowercase() == file_ext.to_lowercase() {
                    return Ok(loader);
                }
            }
        }

        Err(anyhow!(
            "Failed to find suitable saver for extension: {}",
            file_ext
        ))
    }
    pub fn save<T: Asset>(&self, handle: &Handle<T>) -> anyhow::Result<()> {
        let asset_db = self.asset_db.read();
        let path = asset_db
            .handle_to_path(&handle.clone_erased_as_internal())
            .unwrap();
        let path = self.asset_dir.read().join(path);

        let saver = self.get_saver::<T>(
            path.extension()
                .expect("No extension! Cannot guess file type for saving"),
        )?;

        let asset_pool = self.read_assets::<T>().unwrap();

        let asset = asset_pool
            .get(handle)
            .expect("Cannot save! Asset doesn't exist");

        let mode = Mode {
            create: true,
            write: true,
            truncate: true,
            create_new: false,
            read: false,
            append: false,
        };
        let (temp_path, temp_file) = self.io.create_temp_file(&path, &mode)?;

        let mut context = SaveContext::new(asset);
        {
            let mut writer = BufWriter::new(temp_file);
            saver.save(&mut context, &mut writer)?;
            writer.flush()?;
        }
        self.io.rename_file(&temp_path, &path)?;

        self.unsaved.lock().save(&handle.clone_erased_as_internal());

        Ok(())
    }
    pub fn save_all<T: Asset>(&self, only_unsaved: bool) -> anyhow::Result<()> {
        let asset_db = self.asset_db().read();
    
        let Some(records) = asset_db.records_by_type::<T>() else { return Ok(()) };

        for record in records  {
            let uuid = &record.uuid;
            let handle = asset_db.uuid_to_handle(uuid);
            let Some(handle) = handle else {continue};

            if only_unsaved && !self.unsaved.lock().contains(handle) {continue};

            let typed_handle = handle.clone_typed::<T>().unwrap();
            
            if !self.get_saver::<T>(record.path.extension().unwrap()).is_ok() {
                log::debug!("Not saving anything for type: {} as it has no registered savers", type_name::<T>());
                return Ok(());
            }
            
            self.save(&typed_handle)?;
        }

        Ok(())
    }
    pub fn create<T: Asset>(
        &self,
        path: impl AsRef<Path>,
        asset: T,
        settings: Option<T::Settings>,
    ) -> anyhow::Result<Handle<T>> {
        let asset_path = path.as_ref();
        //let asset_path_abs = self.asset_dir.read().join(&asset_path);
        let mut pool = self.write_assets::<T>().unwrap();

        let handle = pool.insert(asset);
        let erased_handle = handle.clone_erased_as_internal();
        self.asset_db.write().create_or_update_record::<T>(
            &erased_handle,
            asset_path,
            settings.unwrap_or_default(),
            LoadStatus::Loaded
        );

        self.unsaved.lock().add_unsaved(erased_handle);
        Ok(handle)
    }
    pub fn mark_unsaved<T: Asset>(&self, handle: &Handle<T>) {
        self.unsaved.lock().add_unsaved(handle.clone_erased_as_internal())
    }
    pub fn rename(&self, path: impl AsRef<Path>, new_path: impl AsRef<Path>) -> anyhow::Result<()> {
        self.asset_db()
            .write()
            .rename_record(path.as_ref(), new_path.as_ref())
    }
    fn queue_update<T: Asset>(&self) {
        let mut asset_db = self.asset_db.write();

        for result in self.load_queue.recv::<T>() {
            match result.result {
                Ok((data, dependencies)) => {
                    let all_deps_loaded = dependencies.iter().all(|dependency| {
                        asset_db.status(dependency.into()) == Some(LoadStatus::Loaded)
                    });

                    if all_deps_loaded {
                        let handle = result.handle;
                        let mut pool = self.write_assets::<T>().unwrap();
                        pool.insert_with_handle(handle.index(), data);

                        log::info!(
                            "Loaded {:?}",
                            asset_db.handle_to_path(&handle).unwrap()
                        );

                        let load_status = asset_db.status_mut(&handle.into()).unwrap();

                        *load_status = LoadStatus::Loaded;

                    } else {
                        self.load_queue
                            .send(LoadResult {
                                handle: result.handle,
                                result: Ok((data, dependencies)),
                            })
                            .expect("Failed to set send load result");
                    }
                }
                Err(err) => {
                    log::error!("{}", err);
                    let load_status = asset_db.status_mut(&result.handle.into()).unwrap();
                    *load_status = LoadStatus::Failed;
                }
            }
        }
    }
    pub fn update<T: Asset>(&self) {
        fn remove_unused_handle(asset_db: &mut AssetDB, handle: &(TypeId, usize)) {
            asset_db.remove_handle_by_ix(&handle);
        }
        self.write_assets::<T>().unwrap()
        .garbage_collect(|index| {
            let mut asset_db = self.asset_db().write();
            remove_unused_handle(&mut asset_db, &(TypeId::of::<T>(), index));
        });
        
        self.queue_update::<T>()
    }
    #[cfg(feature = "serialize")]
    pub fn set_asset_dir(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref();

        assert!(path.is_dir());
        assert!(path.is_absolute());

        let old_path = self.asset_dir.upgradable_read();
        if path != &*old_path {
            *parking_lot::RwLockUpgradableReadGuard::upgrade(old_path) = path.to_owned();
            
            self.load_db()
        } else {
            Ok(())
        }
    }
    #[cfg(feature = "serialize")]
    pub fn get_asset_dir(&self) -> PathBuf {
        self.asset_dir.read().clone()
    }
    #[cfg(feature = "serialize")]
    pub fn save_db(&self) -> anyhow::Result<()> {
        let path = self.asset_dir.read().join("assets.db");
        let io = &self.io;

        if path.exists() {
            let mut path_olded = path.clone();
            path_olded.set_extension("db.old");

            io.rename_file(&path, &path_olded)?;
        }
        let writer = io.write_file(&path, &Mode::create_and_write_and_truncate())?;

        let mut serde_yaml = serde_yaml::Serializer::new(writer);

        use serde::Serialize;
        self.asset_db
            .write()
            .as_serializable(&self.any_serde)
            .serialize(&mut serde_yaml)?;

        Ok(())
    }
    #[cfg(feature = "serialize")]
    pub fn load_db(&self) -> anyhow::Result<()> {
        let path = self.asset_dir.read().join("assets.db");
        let io = &self.io;

        if path.exists() {
            let reader = io.read_file(&path, &Mode::read_only())?;
            let deserializer = serde_yaml::Deserializer::from_reader(reader);
            let asset_db = AssetDB::deserialize(deserializer, &self.any_serde)?;
            
            *self.asset_db().write() = asset_db;
        }
        Ok(())
    }
    pub fn status(&self, handle: &ErasedHandle) -> Option<LoadStatus> {
        self.asset_db.read().status(handle)
    }
}
impl Drop for AssetManagerInner {
    fn drop(&mut self) {
        log::debug!("Dropping Asset Manager");
    }
}
pub struct AssetManagerBuilder {
    thread_pool: Option<Arc<ThreadPool>>,
    asset_pools: HashMap<TypeId, DynAssetPool>,
    asset_db: RwLock<AssetDB>,
    loaders: HashMap<TypeId, Vec<Arc<dyn Loader>>>,
    savers: HashMap<TypeId, Vec<Arc<dyn Saver>>>,
    load_queue: LoadQueue,
    //untyped_loaders: HashMap<TypeId, fn() -> ErasedHandle>,
    io: Option<Arc<dyn IO>>,
    asset_dir: Option<PathBuf>,
    #[cfg(feature = "serialize")]
    any_serde: AnySerde,
}
impl AssetManagerBuilder {
    pub fn new() -> Self {
        Self {
            thread_pool: None,
            asset_pools: HashMap::new(),
            asset_db: RwLock::new(AssetDB::new()),
            loaders: HashMap::new(),
            savers: HashMap::new(),
            load_queue: LoadQueue::new(),
            //untyped_loaders: HashMap::new(),
            io: None,
            asset_dir: None,
            #[cfg(feature = "serialize")]
            any_serde: AnySerde::new(),
        }
    }
    pub fn thread_pool(&mut self, pool: &Arc<ThreadPool>) -> &mut Self {
        self.thread_pool = Some(pool.clone());

        self
    }
    pub fn io(&mut self, io: &Arc<dyn IO>) -> &mut Self {
        self.io = Some(io.clone());

        self
    }
    pub fn register_asset_type<T: Asset>(&mut self) -> &mut Self {
        let existing = self
            .asset_pools
            .insert(TypeId::of::<T>(), DynAssetPool::new::<T>());

        if existing.is_some() {
            panic!("Asset Type: {} already registered", type_name::<T>());
        }

        self.loaders.insert(TypeId::of::<T>(), Vec::new());
        self.savers.insert(TypeId::of::<T>(), Vec::new());

        #[cfg(feature = "serialize")]
        self.any_serde.register_type::<T::Settings>();
        self.load_queue.register_asset_type::<T>();

        self
    }
    pub fn register_loader<T: Asset, L: Loader>(&mut self, loader: L) -> &mut Self {
        let loaders = self
            .loaders
            .get_mut(&TypeId::of::<T>())
            .expect("Asset type not registered");
        loaders.push(Arc::new(loader));

        self
    }
    pub fn register_saver<T: Asset, S: Saver>(&mut self, saver: S) -> &mut Self {
        let savers = self
            .savers
            .get_mut(&TypeId::of::<T>())
            .expect("Asset type not registered");
        savers.push(Arc::new(saver));

        self
    }
    #[cfg(feature = "serialize")]
    pub fn set_asset_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.asset_dir = Some(path.as_ref().to_owned());

        self
    }
    pub fn build(self) -> anyhow::Result<AssetManager> {
        let io = self.io.unwrap_or(Arc::new(PhysicalIO));
        let thread_pool = self
            .thread_pool
            .unwrap_or(Arc::new(rayon::ThreadPoolBuilder::new().build()?));
        let asset_db = self.asset_db;
        let asset_pools = self.asset_pools;
        let loaders = self.loaders;
        let savers = self.savers;
        let load_queue = self.load_queue;

        #[cfg(feature = "serialize")]
        let any_serde = self.any_serde;
        let asset_manager = AssetManagerInner {
            thread_pool,
            io,
            asset_db,
            asset_pools,
            loaders,
            savers,
            #[cfg(feature = "serialize")]
            any_serde,
            load_queue: Arc::new(load_queue),
            asset_dir: RwLock::new(PathBuf::new()),
            unsaved: Mutex::new(Unsaved::default()),
        };
        let asset_manager = AssetManager::new(asset_manager);
              
        #[cfg(feature = "serialize")]
        let asset_dir = self.asset_dir.unwrap_or(std::env::current_dir()?);
        #[cfg(feature = "serialize")]
        asset_manager.set_asset_dir(asset_dir)?;

        Ok(asset_manager)
    }
}
#[derive(Clone)]
pub struct AssetManager {
    inner: Arc<AssetManagerInner>,
}

impl AssetManager {
    pub fn builder() -> AssetManagerBuilder {
        AssetManagerBuilder::new()
    }
    fn new(inner: AssetManagerInner) -> Self {
        let manager = Self {
            inner: Arc::new(inner),
        };

        init_asset_manager(manager.clone());

        manager
    }

    pub fn read_assets<T: Asset>(&self) -> Option<PoolRef<T>> {
        self.inner.read_assets()
    }
    pub fn write_assets<T: Asset>(&self) -> Option<PoolMut<T>> {
        self.inner.write_assets()
    }

    pub fn asset_db(&self) -> &RwLock<AssetDB> {
        self.inner.asset_db()
    }
    pub fn  load<T: Asset>(
        &self,
        path: impl AsPath,
        settings: Option<T::Settings>,
        reload: bool,
    ) -> anyhow::Result<Handle<T>> {
        self.inner.load(path, settings, reload)
    }
    pub fn load_lazy<T: Asset>(
        &self,
        path: impl AsPath,
        settings: Option<T::Settings>,
    ) -> anyhow::Result<Handle<T>> {
        self.inner.load_lazy(path, settings)
    }
    // pub fn request_load<T: Asset>(
    //     &self,
    //     handle: &Handle<T>,
    //     settings: Option<T::Settings>,
    //     reload: bool,
    // ) -> anyhow::Result<()> {
    //     self.inner.request_load(&handle, settings, reload)
    // }
    pub fn wait_for_load<T: Asset>(&self, handle: &Handle<T>) -> LoadStatus {
        self.inner.wait_for_load(handle)
    }
    pub fn save<T: Asset>(&self, handle: &Handle<T>) -> anyhow::Result<()> {
        self.inner.save::<T>(handle)
    }
    pub fn save_all<T: Asset>(&self, only_unsaved: bool) -> anyhow::Result<()> {
        self.inner.save_all::<T>(only_unsaved)
    }
    pub fn create<T: Asset>(
        &self,
        save_path: impl AsRef<Path>,
        asset: T,
        settings: Option<T::Settings>,
    ) -> anyhow::Result<Handle<T>> {
        self.inner.create::<T>(save_path, asset, settings)
    }
    pub fn mark_unsaved<T: Asset>(&self, handle: &Handle<T>) {
        self.inner.mark_unsaved(handle)
    }
    pub fn rename(&self, path: impl AsRef<Path>, new_path: impl AsRef<Path>) -> anyhow::Result<()> {
        self.inner.rename(path, new_path)
    }
    pub fn update<T: Asset>(&self) {
        self.inner.update::<T>()
    }
    #[cfg(feature = "serialize")]
    pub fn set_asset_dir(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        self.inner.set_asset_dir(path)
    }
    #[cfg(feature = "serialize")]
    pub fn get_asset_dir(&self) -> PathBuf {
        self.inner.get_asset_dir()
    }
    #[cfg(feature = "serialize")]
    pub fn save_db(&self) -> anyhow::Result<()> {
        self.inner.save_db()
    }
    #[cfg(feature = "serialize")]
    pub fn load_db(&self) -> anyhow::Result<()> {
        self.inner.load_db()
    }
    pub fn status(&self, handle: &ErasedHandle) -> Option<LoadStatus> {
        self.inner.status(handle)
    }
}

static ASSET_MANAGER: OnceCell<Weak<AssetManagerInner>> = OnceCell::new();

pub(crate) fn init_asset_manager(asset_manager: AssetManager) {
    if ASSET_MANAGER.get().is_some() {
        panic!("Asset Manager has already been initialized");
    }

    ASSET_MANAGER.get_or_init(|| Arc::downgrade(&asset_manager.inner));
}

pub(crate) fn get_asset_manager() -> AssetManager {
    let inner = ASSET_MANAGER
        .get()
        .expect("AssetManager has not been initialized");
    AssetManager {
        inner: inner.upgrade().expect("Asset Manager has been dropped"),
    }
}

#[cfg(feature = "serialize")]
#[test]
fn txt_save_and_load() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new().init().unwrap();

    use std::io::Read;
    #[derive(Debug, type_uuid::TypeUuid)]
    #[uuid = "5692c7f0-faa2-44ac-b1fd-431081e2372f"]
    struct TxtFile {
        contents: String,
    }

    impl Asset for TxtFile {
        type Settings = ();
    }

    struct TxtSaverLoader;

    impl Loader for TxtSaverLoader {
        fn extensions(&self) -> &[&str] {
            &["txt"]
        }
        fn load(&self, ctx: &mut LoadContext) -> anyhow::Result<()> {
            let mut contents = String::new();
            ctx.reader().read_to_string(&mut contents)?;

            ctx.set_asset(TxtFile { contents });
            Ok(())
        }
    }
    impl Saver for TxtSaverLoader {
        fn extensions(&self) -> &[&str] {
            &["txt"]
        }

        fn save(
            &self,
            context: &mut SaveContext,
            writer: &mut dyn std::io::Write,
        ) -> anyhow::Result<()> {
            writer.write(context.get_asset::<TxtFile>().contents.as_bytes())?;
            Ok(())
        }
    }

    let mut ass_man = AssetManager::builder();
    ass_man.register_asset_type::<TxtFile>();
    ass_man.register_loader::<TxtFile, TxtSaverLoader>(TxtSaverLoader);
    ass_man.register_saver::<TxtFile, TxtSaverLoader>(TxtSaverLoader);

    let ass_man = ass_man.build()?;

    ass_man.set_asset_dir(std::env::current_dir()?)?;

    let test = ass_man.load::<TxtFile>(Path::new("test.txt"), None, false)?;
    let test1 = ass_man.load::<TxtFile>(Path::new("DoesNotExist.txt"), None, false)?;

    assert!(ass_man.status(&test.clone_erased()) == Some(LoadStatus::Loading));
    assert!(ass_man.status(&test1.clone_erased()) == Some(LoadStatus::Loading));

    let test_erased = test.clone_erased();
    let test1_erased = test1.clone_erased();

    while ass_man.status(&test_erased) != Some(LoadStatus::Loaded)
        || ass_man.status(&test1_erased) != Some(LoadStatus::Failed)
    {
        ass_man.update::<TxtFile>();
    }

    let mut pool = ass_man.write_assets::<TxtFile>().unwrap();

    let text = pool.get_mut(&test).unwrap();
    text.contents.push_str("s");

    log::debug!("{:#?}", pool.get(&test));
    drop(pool);

    ass_man.save(&test)?;

    ass_man.update::<TxtFile>();

    ass_man.save_db()?;
    Ok(())
}


#[test]
fn memory_leak() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new().init().unwrap();

    use std::io::Read;
    #[derive(Debug, type_uuid::TypeUuid)]
    #[uuid = "5692c7f0-faa2-44ac-b1fd-431081e2372f"]
    struct TxtFile {
        _contents: String,
    }

    impl Asset for TxtFile {
        type Settings = ();
    }

    struct TxtSaverLoader;

    impl Loader for TxtSaverLoader {
        fn extensions(&self) -> &[&str] {
            &["txt"]
        }
        fn load(&self, ctx: &mut LoadContext) -> anyhow::Result<()> {
            let mut contents = String::new();
            ctx.reader().read_to_string(&mut contents)?;

            ctx.set_asset(TxtFile { _contents: contents });
            Ok(())
        }
    }

    let mut ass_man = AssetManager::builder();
    ass_man.register_asset_type::<TxtFile>();
    ass_man.register_loader::<TxtFile, TxtSaverLoader>(TxtSaverLoader);

    let ass_man = ass_man.build()?;

    //ass_man.set_asset_dir(std::env::current_dir()?)?;
    let test = ass_man.load::<TxtFile>(Path::new("test.txt"), None, false)?;

    assert!(ass_man.status(&test.clone_erased()) == Some(LoadStatus::Loading));

    let test_erased = test.clone_erased();

    while ass_man.status(&test_erased) != Some(LoadStatus::Loaded)
    {
        ass_man.update::<TxtFile>();
    }

    drop(test_erased);

    //let pool = ass_man.write_assets::<TxtFile>().unwrap();

    assert_eq!(test.strong_count(), 1);

    // drop(pool);

    Ok(())
}