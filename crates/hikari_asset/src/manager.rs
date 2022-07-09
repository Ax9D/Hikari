use crate::{
    handle::{ErasedHandle, Handle, HandleAllocator},
    meta::MetaData,
    Asset, AssetPool, LoadContext, Loader, Source,
};
use anyhow::anyhow;
use parking_lot::{Mutex, RwLock};
use rayon::ThreadPool;
use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LoadStatus {
    Loading,
    Loaded,
    Unloaded,
    Failed,
    Waiting,
}
#[derive(Default)]
struct AssetDB {
    uuid_to_handle: HashMap<Uuid, ErasedHandle>,
    uuid_to_path: HashMap<Uuid, PathBuf>,
    handle_to_uuid: HashMap<ErasedHandle, Uuid>,
    uuid_to_statues: HashMap<Uuid, LoadStatus>,
}
impl AssetDB {
    pub fn create_with_status(
        &mut self,
        uuid: Uuid,
        path: PathBuf,
        erased_handle: ErasedHandle,
        status: LoadStatus,
    ) {
        self.uuid_to_handle.insert(uuid, erased_handle.clone());
        self.handle_to_uuid.insert(erased_handle, uuid);
        self.uuid_to_path.insert(uuid, path);
        self.uuid_to_statues.insert(uuid, status);
    }
    pub fn set_status(&mut self, uuid: Uuid, status: LoadStatus) {
        *self
            .uuid_to_statues
            .get_mut(&uuid)
            .expect("Asset was not registered") = status;
    }
    pub fn get_status(&self, handle: &ErasedHandle) -> Option<LoadStatus> {
        let uuid = self.handle_to_uuid.get(&handle)?;
        let status = self.uuid_to_statues.get(uuid).cloned()?;
        Some(status)
    }
    pub fn get_uuid(&self, handle: &ErasedHandle) -> Uuid {
        self.handle_to_uuid.get(handle).unwrap().clone()
    }
    pub fn get_path(&self, handle: &ErasedHandle) -> PathBuf {
        self.uuid_to_path
            .get(&self.get_uuid(handle))
            .unwrap()
            .clone()
    }
}
struct LoadResult<T: Asset> {
    asset: Result<T, anyhow::Error>,
    handle: Handle<T>,
    meta: MetaData<T>,
}

struct LoadState<T: Asset> {
    handle_allocator: Arc<HandleAllocator>,
    load_send: flume::Sender<LoadResult<T>>,
    load_recv: flume::Receiver<LoadResult<T>>,
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

struct AssetManagerInner {
    db: Mutex<AssetDB>,
    loaders: HashMap<TypeId, Vec<Arc<dyn Loader>>>,
    thread_pool: Arc<ThreadPool>,
    load_states: HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>,
}
impl AssetManagerInner {
    pub fn with_threadpool(thread_pool: Arc<ThreadPool>) -> Self {
        Self {
            thread_pool,
            db: Default::default(),
            loaders: Default::default(),
            load_states: Default::default(),
        }
    }
    fn load_state<T: Asset>(&self) -> anyhow::Result<&LoadState<T>> {
        Ok(self
            .load_states
            .get(&TypeId::of::<T>())
            .ok_or_else(|| anyhow::anyhow!("Asset type {} not registered", type_name::<T>()))?
            .downcast_ref::<LoadState<T>>()
            .unwrap())
    }
    fn load_task<T: Asset>(
        &self,
        source: Source,
        meta: MetaData<T>,
        handle: &Handle<T>,
        loader: Arc<dyn Loader>,
        ass_man: AssetManager,
    ) -> anyhow::Result<()> {
        let load_state = self.load_state()?;

        let sender = load_state.load_send.clone();
        let mut load_context = LoadContext::new::<T>(source, meta.settings.clone(), ass_man);
        let loader = loader.clone();
        let handle_clone = handle.clone();

        self.thread_pool.spawn(move || {
            let result = loader.load(&mut load_context);
            let asset = if result.is_ok() {
                Ok(load_context
                    .take_asset()
                    .expect("Asset needs to be set after loading!"))
            } else {
                Err(anyhow::anyhow!(
                    "Failed to load asset {}: {}",
                    meta.source_path.display(),
                    result.err().unwrap()
                ))
            };
            if sender
                .send(LoadResult {
                    asset,
                    handle: handle_clone,
                    meta,
                })
                .is_err()
            {
                log::error!("Failed to enqueue load task");
            }
        });

        Ok(())
    }
    fn load_with_loader<T: Asset>(
        &self,
        source: Source,
        settings: Option<T::Settings>,
        loader: &Arc<dyn Loader>,
        ass_man: AssetManager,
    ) -> anyhow::Result<Handle<T>> {
        let mut meta = MetaData::<T>::for_file(&source.path()).unwrap_or_else(|| MetaData {
            source_path: source.path().to_owned(),
            is_standalone: source.is_filesystem(),
            uuid: Uuid::new_v4(),
            settings: settings.clone().unwrap_or_default(),
        });

        //If settings are specified override those obtained from the metadata
        if let Some(settings) = settings {
            meta.settings = settings;
        }

        let mut db = self.db.lock();
        let erased = db.uuid_to_handle.get(&meta.uuid);

        let handle = match erased.cloned() {
            Some(handle) => {
                if db.get_status(&handle) == Some(LoadStatus::Waiting) && !source.is_filesystem() {
                    db.set_status(meta.uuid, LoadStatus::Loading);

                    let typed = handle.clone_typed().unwrap();

                    self.load_task(source, meta, &typed, loader.clone(), ass_man)?;
                }
                handle
            }
            None => {
                let load_state = self.load_state::<T>()?;
                let allocated_handle = load_state.handle_allocator.allocate();

                let erased_handle = allocated_handle.clone_erased();

                let status = if source.is_filesystem() {
                    LoadStatus::Loading
                } else {
                    LoadStatus::Waiting
                };

                db.create_with_status(
                    meta.uuid.clone(),
                    meta.source_path.clone(),
                    erased_handle,
                    status,
                );

                self.load_task(source, meta, &allocated_handle, loader.clone(), ass_man)?;
                allocated_handle.clone_erased()
            }
        };

        Ok(handle.into_typed().expect("Handle type mismatch"))
    }
    pub fn register_asset<T: Asset>(&mut self, pool: &AssetPool<T>) {
        self.loaders.insert(TypeId::of::<T>(), vec![]);
        self.load_states.insert(
            TypeId::of::<T>(),
            Box::new(LoadState::<T>::new(pool.handle_allocator().clone())),
        );
    }
    pub fn add_loader<T: Asset, L: Loader>(&mut self, loader: L) {
        let loaders = self
            .loaders
            .get_mut(&TypeId::of::<T>())
            .expect("Asset type not registered");
        loaders.push(Arc::new(loader));
    }
    fn get_loader<T: Asset>(&self, path: &Path) -> anyhow::Result<&Arc<dyn Loader>> {
        let file_ext = path
            .extension()
            .ok_or_else(|| anyhow!("Couldn't determine file extension"))?;
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
    pub fn load<T: Asset>(&self, path: &Path, ass_man: AssetManager) -> anyhow::Result<Handle<T>> {
        let loader = self.get_loader::<T>(path)?;
        let handle =
            self.load_with_loader::<T>(Source::FileSystem(path.to_owned()), None, loader, ass_man)?;

        log::info!("Trying to load: {:#?}", path);
        Ok(handle)
    }
    pub fn load_with_settings<T: Asset>(
        &self,
        path: &Path,
        settings: T::Settings,
        ass_man: AssetManager,
    ) -> anyhow::Result<Handle<T>> {
        let loader = self.get_loader::<T>(path)?;
        let handle = self.load_with_loader::<T>(
            Source::FileSystem(path.to_owned()),
            Some(settings),
            loader,
            ass_man,
        )?;

        log::info!("Trying to load: {:#?}", path);
        Ok(handle)
    }
    pub fn load_with_data<T: Asset>(
        &self,
        name: &Path,
        data: Vec<u8>,
        settings: T::Settings,
        ass_man: AssetManager,
    ) -> anyhow::Result<Handle<T>> {
        let loader = self.get_loader::<T>(Path::new(name))?;
        let handle = self.load_with_loader::<T>(
            Source::Data(name.to_owned(), data),
            Some(settings),
            loader,
            ass_man,
        )?;

        log::info!("Trying to load data using apparent path: {:#?}", name);

        Ok(handle)
    }
    pub fn update<T: Asset>(&self, pool: &mut AssetPool<T>) -> anyhow::Result<()> {
        let load_state = self
            .load_states
            .get(&TypeId::of::<T>())
            .expect("Asset type not registed")
            .downcast_ref::<LoadState<T>>()
            .unwrap();

        for result in load_state.load_recv.try_iter() {
            let meta = result.meta;
            match result.asset {
                Ok(asset) => {
                    log::info!("Loaded {:#?}", meta.source_path);
                    self.db.lock().set_status(meta.uuid, LoadStatus::Loaded);
                    pool.insert(result.handle.index(), asset);

                    meta.save()?;
                }
                Err(err) => {
                    log::error!("Failed to load asset {:#?}: {}", meta.source_path, err);
                    self.db.lock().set_status(meta.uuid, LoadStatus::Failed);
                }
            }
        }
        Ok(())
    }
    pub fn load_status(&self, handle: &ErasedHandle) -> Option<LoadStatus> {
        self.db.lock().get_status(handle)
    }
    pub fn get_uuid<T: Asset>(&self, handle: &Handle<T>) -> Uuid {
        self.db.lock().get_uuid(&handle.clone_erased())
    }
    pub fn get_path<T: Asset>(&self, handle: &Handle<T>) -> PathBuf {
        self.db.lock().get_path(&handle.clone_erased())
    }
}
#[derive(Clone)]
pub struct AssetManager {
    inner: Arc<RwLock<AssetManagerInner>>,
}
impl AssetManager {
    pub fn with_threadpool(threadpool: Arc<ThreadPool>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(AssetManagerInner::with_threadpool(threadpool))),
        }
    }
    pub fn register_asset<T: Asset>(&mut self, pool: &AssetPool<T>) {
        self.inner.write().register_asset(pool)
    }
    pub fn add_loader<T: Asset, L: Loader>(&mut self, loader: L) {
        self.inner.write().add_loader::<T, L>(loader)
    }
    pub fn load<T: Asset>(&self, path: &Path) -> anyhow::Result<Handle<T>> {
        self.inner.read().load(path, self.clone())
    }
    pub fn load_with_settings<T: Asset>(
        &self,
        path: &Path,
        settings: T::Settings,
    ) -> anyhow::Result<Handle<T>> {
        self.inner
            .read()
            .load_with_settings(path, settings, self.clone())
    }
    pub fn load_with_data<T: Asset>(
        &self,
        name: &Path,
        data: Vec<u8>,
        settings: T::Settings,
    ) -> anyhow::Result<Handle<T>> {
        self.inner
            .read()
            .load_with_data(name, data, settings, self.clone())
    }
    pub fn update<T: Asset>(&self, pool: &mut AssetPool<T>) -> anyhow::Result<()> {
        self.inner.read().update(pool)
    }
    pub fn load_status(&self, handle: &ErasedHandle) -> Option<LoadStatus> {
        self.inner.read().load_status(handle)
    }

    pub fn get_uuid<T: Asset>(&self, handle: &Handle<T>) -> Uuid {
        self.inner.read().get_uuid(handle)
    }
    pub fn get_path<T: Asset>(&self, handle: &Handle<T>) -> PathBuf {
        self.inner.read().get_path(handle)
    }
}
#[test]
fn txt_load() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new().init().unwrap();

    #[derive(Debug)]
    struct TxtFile {
        contents: String,
    }

    impl Asset for TxtFile {
        type Settings = ();
    }

    struct TxtLoader;

    impl Loader for TxtLoader {
        fn extensions(&self) -> &[&str] {
            &["txt"]
        }
        fn load(&self, ctx: &mut LoadContext) -> anyhow::Result<()> {
            let path = ctx.source().path();
            let contents = std::fs::read_to_string(path)?;
            ctx.set_asset(TxtFile { contents });
            Ok(())
        }
    }
    use rayon::ThreadPoolBuilder;
    let tp = Arc::new(ThreadPoolBuilder::new().build()?);
    let mut pool = AssetPool::default();
    let mut ass_man = AssetManager::with_threadpool(tp);
    ass_man.register_asset::<TxtFile>(&pool);
    ass_man.add_loader::<TxtFile, TxtLoader>(TxtLoader);

    let test = ass_man.load::<TxtFile>(Path::new("test.txt"))?;
    let test1 = ass_man.load::<TxtFile>(Path::new("DoesNotExist.txt"))?;

    assert!(ass_man.load_status(&test.clone_erased()) == Some(LoadStatus::Loading));
    assert!(ass_man.load_status(&test1.clone_erased()) == Some(LoadStatus::Loading));

    ass_man.update::<TxtFile>(&mut pool)?;

    assert!(ass_man.load_status(&test.clone_erased()) == Some(LoadStatus::Loaded));
    assert!(ass_man.load_status(&test1.clone_erased()) == Some(LoadStatus::Failed));

    log::debug!("{:#?}", pool.get(&test));
    Ok(())
}
