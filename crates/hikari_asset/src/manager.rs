use crate::{Asset, AssetDB, PhysicalIO, DynAssetPool, ErasedHandle, Handle, LoadContext,
    Loader, Mode, PoolMut, PoolRef, IO, SaveContext, Saver,
};
#[cfg(feature = "serialize")]
use crate::{
    serialize::AnySerde};
    
use anyhow::anyhow;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use rayon::ThreadPool;
use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc, io::{BufWriter, Write}, ffi::OsStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadStatus {
    Loaded,
    Loading,
    Unloaded,
    Failed,
}
struct LoadResult<T: Asset> {
    result: anyhow::Result<T>,
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
    pub fn recv<T: Asset>(&self) -> flume::TryIter<LoadResult<T>> {
        let any = self.queues.get(&TypeId::of::<T>()).unwrap();
        any.downcast_ref::<QueueEnds<T>>().unwrap().1.try_iter()
    }
}
struct AssetManagerInner {
    asset_pools: HashMap<TypeId, DynAssetPool>,
    asset_db: RwLock<AssetDB>,
    loaders: HashMap<TypeId, Vec<Arc<dyn Loader>>>,
    savers: HashMap<TypeId, Vec<Arc<dyn Saver>>>,
    load_statuses: RwLock<HashMap<ErasedHandle, LoadStatus>>,
    thread_pool: Arc<ThreadPool>,
    io: Arc<dyn IO>,
    load_queue: Arc<LoadQueue>,
    #[cfg(feature = "serialize")]
    any_serde: AnySerde,
    asset_dir: RwLock<PathBuf>,
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
                if extension == file_ext {
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
        abs_path: PathBuf,
        rel_path: PathBuf,
        settings: T::Settings,
        reload: bool,
        io: Arc<dyn IO>,
        loader: Arc<dyn Loader>,
    ) -> anyhow::Result<T> {
        let reader = io.read_file(&abs_path, &Mode::read_only())?;
        let mut ctx =
            LoadContext::new::<T>(abs_path, rel_path, io.clone(), reader, settings, reload, get_asset_manager().clone());
        loader.load(&mut ctx)?;

        let asset = ctx.take_asset::<T>().expect("Asset not set during loading");

        Ok(asset)
    }
    fn trigger_load<T: Asset>(
        &self,
        handle: &ErasedHandle,
        path: &Path,
        settings: &T::Settings,
        reload: bool,
    ) -> anyhow::Result<()> {
        self.load_statuses
            .write()
            .insert(handle.clone(), LoadStatus::Loading);

        let loader = self.get_loader::<T>(path)?.clone();

        let io = self.io.clone();
        let abs_path = self.asset_dir.read().join(path);
        let rel_path = path.to_owned();
        let settings = settings.clone();
        let load_queue = self.load_queue.clone();
        let handle = handle.clone();
        self.thread_pool.spawn(move || {
            let result = Self::load_task::<T>(abs_path, rel_path, settings, reload, io, loader);

            let load_result = LoadResult { result, handle };
            load_queue
                .send::<T>(load_result)
                .expect("Failed to send load result");
        });
        Ok(())
    }
    pub fn load<T: Asset>(
        &self,
        path: impl AsRef<Path>,
        settings: Option<T::Settings>,
        reload: bool,
    ) -> anyhow::Result<Handle<T>> {
        let path = path.as_ref();

        if !path.is_relative() {
            return Err(anyhow::anyhow!(
                "Absolute paths are not supported. Use a path relative to your asset directory"
            ));
        }

        let mut db = self.asset_db.write();
        if let Some((handle, record)) = db.path_to_handle_and_record(path) {
            if let Some(settings) = settings {
                *record.settings_mut::<T>() = settings;
            }

            let settings = record.settings::<T>();

            let load_statuses = self.load_statuses.read();
            let load_status = *load_statuses.get(handle).unwrap();
            drop(load_statuses);
            match load_status {
                LoadStatus::Unloaded | LoadStatus::Failed => {
                    self.trigger_load::<T>(handle, path, &settings, reload)?;
                }
                _ if reload => {
                    self.trigger_load::<T>(handle, path, &settings, reload)?;
                }
                _ => {}
            }

            return Ok(handle.clone_strong().clone_typed::<T>().unwrap());
        }

        //Fresh load
        let asset_pool = self
            .asset_pools
            .get(&TypeId::of::<T>())
            .ok_or(anyhow::anyhow!("Unregistered asset type"))?;
        let asset_pool = asset_pool.read::<T>();
        let handle_allocator = asset_pool.handle_allocator();
        let handle = handle_allocator.allocate::<T>();
        let erased_handle = handle.clone_erased_as_weak();

        let settings = settings.unwrap_or(T::Settings::default());
        db.assign_handle::<T>(&erased_handle, path.to_owned(), settings.clone());
        self.trigger_load::<T>(&erased_handle, path, &settings, reload)?;

        Ok(handle)
    }

    fn get_saver<T: Asset>(&self, extension: &OsStr) -> anyhow::Result<&Arc<dyn Saver>> {
        let file_ext = extension.to_str().unwrap();
        let savers = self
            .savers
            .get(&TypeId::of::<T>())
            .ok_or_else(|| anyhow!("Saver for asset: {} not found", type_name::<T>()))?;

        for loader in savers {
            for &extension in loader.extensions() {
                if extension == file_ext {
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
        let path = asset_db.handle_to_path(&handle.clone_erased_as_weak()).unwrap();
        let path = self.asset_dir.read().join(path);

        let saver = self.get_saver::<T>(
            path.extension()
                .expect("No extension! Cannot guess file type for saving"),
        )?;

        let asset_pool = self.read_assets::<T>().unwrap();
        
        let asset = asset_pool
            .get(handle)
            .expect("Cannot save! Asset doesn't exist");
            
        let (temp_path, temp_file) = self.io.create_temp_file(&path, &Mode::create_and_write())?;
            
        let mut context = SaveContext::new(asset);
        {
            let mut writer = BufWriter::new(temp_file);
            saver.save(&mut context, &mut writer)?;
            writer.flush()?;
        }
        self.io.rename_file(&temp_path, &path)?;

        Ok(())
    }
    pub fn rename(&self, path: impl AsRef<Path>, new_path: impl AsRef<Path>) -> anyhow::Result<()> {
        self.asset_db().write().rename_record(path.as_ref(), new_path.as_ref())
    }
    fn queue_update<T: Asset>(&self) {
        let mut pool = self.write_assets::<T>().unwrap();

        for result in self.load_queue.recv::<T>() {
            let mut load_statuses = self.load_statuses.write();
            let load_status = load_statuses.get_mut(&result.handle).unwrap();

            match result.result {
                Ok(data) => {
                    log::info!(
                        "Loaded {:?}",
                        self.asset_db.read().handle_to_path(&result.handle).unwrap()
                    );

                    let handle = result.handle.into_typed::<T>().unwrap();
                    pool.insert_with_handle(&handle, data);

                    *load_status = LoadStatus::Loaded;
                }
                Err(err) => {
                    log::error!("{}", err);

                    *load_status = LoadStatus::Failed;
                }
            }
        }
    }
    pub fn update<T: Asset>(&self) {
        self.queue_update::<T>()
    }
    pub fn set_asset_dir(&self, path: impl AsRef<Path>) {
        let path = path.as_ref();

        assert!(path.is_dir());
        assert!(path.is_absolute());

        *self.asset_dir.write() = path.to_owned();

        //self.load_db()
    }
    #[cfg(feature = "serialize")]
    pub fn save_db(&self) -> anyhow::Result<()> {
        let path = self.asset_dir.read().join("assets.db");
        let io = &self.io;

        let writer = io.write_file(&path, &Mode::create_and_write())?;

        let mut serde_yaml = serde_yaml::Serializer::new(writer);

        use serde::Serialize;
        self.asset_db
            .read()
            .as_serializable(&self.any_serde)
            .serialize(&mut serde_yaml)?;

        Ok(())
    }
    #[cfg(feature = "serialize")]
    pub fn load_db(&self) -> anyhow::Result<()> {
        let path = self.asset_dir.read().join("assets.db");
        let io = &self.io;

        let reader = io.read_file(&path, &Mode::read_only());

        let reader = match reader {
            Ok(reader) => Ok(reader),
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    *self.asset_db().write() = AssetDB::new();
                    self.save_db()?;

                    return Ok(());
                }
                _ => Err(err),
            },
        }?;

        let deserializer = serde_yaml::Deserializer::from_reader(reader);
        let asset_db = AssetDB::deserialize(deserializer, &self.any_serde)?;

        *self.asset_db().write() = asset_db;
        Ok(())
    }
    pub fn status(&self, handle: &ErasedHandle) -> Option<LoadStatus> {
        self.load_statuses.read().get(handle).copied()
    }
}

impl Drop for AssetManagerInner {
    fn drop(&mut self) {
        #[cfg(feature = "serialize")]
        self.save_db()
            .expect("Failed to save DB before dropping Asset Manager");
    }
}

pub struct AssetManagerBuilder {
    thread_pool: Option<Arc<ThreadPool>>,
    asset_pools: HashMap<TypeId, DynAssetPool>,
    asset_db: RwLock<AssetDB>,
    loaders: HashMap<TypeId, Vec<Arc<dyn Loader>>>,
    savers: HashMap<TypeId, Vec<Arc<dyn Saver>>>,
    load_queue: LoadQueue,
    io: Option<Arc<dyn IO>>,
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
            io: None,
            #[cfg(feature = "serialize")]
            any_serde: AnySerde::new(),
        }
    }
    pub fn thread_pool(&mut self, pool: &Arc<ThreadPool>) {
        self.thread_pool = Some(pool.clone());
    }
    pub fn io(&mut self, io: &Arc<dyn IO>) {
        self.io = Some(io.clone());
    }
    pub fn register_asset_type<T: Asset>(&mut self) {
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
    }
    pub fn register_loader<T: Asset, L: Loader>(&mut self, loader: L) {
        let loaders = self
            .loaders
            .get_mut(&TypeId::of::<T>())
            .expect("Asset type not registered");
        loaders.push(Arc::new(loader));
    }
    pub fn register_saver<T: Asset, S: Saver>(&mut self, saver: S) {
        let savers = self
            .savers
            .get_mut(&TypeId::of::<T>())
            .expect("Asset type not registered");
        savers.push(Arc::new(saver));
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
            load_statuses: RwLock::new(HashMap::new()),
            asset_dir: RwLock::new(PathBuf::new()),
        };

        asset_manager.set_asset_dir(std::env::current_dir()?);

        let asset_manager = AssetManager::new(asset_manager);

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
    pub fn load<T: Asset>(
        &self,
        path: impl AsRef<Path>,
        settings: Option<T::Settings>,
        reload: bool,
    ) -> anyhow::Result<Handle<T>> {
        self.inner.load(path, settings, reload)
    }
    pub fn save<T: Asset>(&self, handle: &Handle<T>) -> anyhow::Result<()> {
        self.inner.save::<T>(handle)
    }
    pub fn rename(&self, path: impl AsRef<Path>, new_path: impl AsRef<Path>) -> anyhow::Result<()> {
        self.inner.rename(path, new_path)
    }
    pub fn update<T: Asset>(&self) {
        self.inner.update::<T>()
    }
    pub fn set_asset_dir(&self, path: impl AsRef<Path>) {
        self.inner.set_asset_dir(path)
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

static ASSET_MANAGER: OnceCell<AssetManager> = OnceCell::new();

pub(crate) fn init_asset_manager(asset_manager: AssetManager) {
    if ASSET_MANAGER.get().is_some() {
        panic!("Asset Manager has already been initialized");
    }

    ASSET_MANAGER.get_or_init(|| asset_manager);
}

pub(crate) fn get_asset_manager() -> &'static AssetManager {
    ASSET_MANAGER
        .get()
        .expect("AssetManager has not been initialized")
}

#[test]
fn txt_save_and_load() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new().init().unwrap();

    use std::io::Read;
    #[derive(Debug)]
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

    ass_man.load_db()?;

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
