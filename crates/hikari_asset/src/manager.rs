use crate::{
    handle::*,
    load::*,
    meta::*,
    path::{self},
    save::*,
    Asset, AssetPool,
};

use anyhow::anyhow;
use parking_lot::RwLock;
use rayon::ThreadPool;
use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    ffi::OsStr,
    io::{BufWriter, Cursor, Read},
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
struct AssetManagerInner {
    db: RwLock<AssetDB>,
    loaders: HashMap<TypeId, Vec<Arc<dyn Loader>>>,
    load_states: HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>,
    savers: HashMap<TypeId, Vec<Arc<dyn Saver>>>,
    //save_states: HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>
    thread_pool: Arc<ThreadPool>,
    asset_dir: PathBuf,
}
impl AssetManagerInner {
    pub fn with_threadpool(thread_pool: Arc<ThreadPool>) -> Self {
        Self {
            thread_pool,
            db: Default::default(),
            loaders: Default::default(),
            load_states: Default::default(),
            savers: Default::default(),
            asset_dir: std::env::current_dir().expect("Failed to get current directory"), // save_states: Default::default()
        }
    }
    pub fn set_asset_dir(&mut self, path: impl AsRef<Path>) {
        assert!(path.as_ref().is_dir());
        self.asset_dir = path.as_ref().to_owned();
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
        let path = source.absolute_path(&self.asset_dir);

        println!("Absolute load path {:#?}", path);

        let loader = loader.clone();
        let handle_clone = handle.clone();

        fn thread_load_task<T: Asset>(
            source: Source,
            path: PathBuf,
            meta: &MetaData<T>,
            loader: Arc<dyn Loader>,
            ass_man: AssetManager,
        ) -> anyhow::Result<T> {
            let reader: Box<dyn Read + Send + Sync + 'static> = match source {
                Source::FileSystem(_) => Box::new(std::fs::File::open(&path)?),
                Source::Data(_, data) => Box::new(Cursor::new(data)),
            };
            let mut load_context =
                LoadContext::new::<T>(path, reader, meta.settings.clone(), ass_man);

            loader.load(&mut load_context)?;

            Ok(load_context
                .take_asset()
                .expect("Asset needs to be set after loading!"))
        }

        self.thread_pool.spawn(move || {
            let result = thread_load_task::<T>(source, path, &meta, loader, ass_man);

            let asset = result.map_err(|err| {
                anyhow::anyhow!("Failed to load asset {}: {}", meta.source.display(), err)
            });
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
        // let referenceable_path = source.referenceable_path();

        // if referenceable_path.is_absolute() {
        //     log::warn!("Trying to load asset from path outside the asset dir, this will cause problems during distribution");
        // }
        println!("{:#?}", source.relative_path());

        let mut meta = MetaData::<T>::for_file(&source.absolute_path(&self.asset_dir))
            .unwrap_or_else(|| MetaData {
                source: source.relative_path().to_owned(),
                is_standalone: source.is_filesystem(),
                uuid: Uuid::new_v4(),
                settings: settings.clone().unwrap_or_default(),
            });

        let mut db = self.db.write();
        //If settings are specified override those obtained from the metadata
        if let Some(settings) = settings {
            meta.settings = settings;
        }

        let erased = db.uuid_to_handle.get(&meta.uuid);

        let handle = match erased.cloned() {
            // Some(handle) => {
            //     match db.get_status(&handle) {
            //         Some(LoadStatus::Loading) => {
            //             handle.clone()
            //         }
            //         Some(LoadStatus::Waiting) if !source.is_filesystem() => {
            //             db.set_status(meta.uuid, LoadStatus::Loading);

            //             let typed = handle.clone_typed().unwrap();

            //             self.load_task(source, meta, &typed, loader.clone(), ass_man)?;
            //             handle
            //         }
            //         _=> {
            //             todo!()
            //         }
            //     }
            // }
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
                    meta.source.clone(),
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

        self.savers.insert(TypeId::of::<T>(), vec![]);
        // self.save_states.insert(
        //     TypeId::of::<T>(),
        // Box::new(SaveState::<T>::new())
        // );
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

    fn get_relative_path(&self, path: &Path) -> PathBuf {
        println!("{}", self.asset_dir.display());
        if path.is_relative() {
            path.to_owned()
        } else {
            path::make_relative(path, &self.asset_dir).unwrap()
        }
        //Err(anyhow::anyhow!("Only paths relative to the cwd({}) are allowed, Please move your asset files to this directory", self.asset_dir.display()))
    }
    pub fn load<T: Asset>(&self, path: &Path, ass_man: AssetManager) -> anyhow::Result<Handle<T>> {
        let path = self.get_relative_path(path);

        let loader = self.get_loader::<T>(&path)?;

        // let metadata = MetaData::<T>::for_file(path).unwrap_or(MetaData {
        //     source_path: path.to_owned(),
        //     is_standalone: path.exists(),
        //     uuid: Uuid::new_v4(),
        //     settings: T::Settings::default(),
        // });

        log::info!("Trying to load: {}", path.display());

        let handle = self.load_with_loader::<T>(Source::FileSystem(path), None, loader, ass_man)?;

        Ok(handle)
    }
    pub fn load_with_settings<T: Asset>(
        &self,
        path: &Path,
        settings: T::Settings,
        ass_man: AssetManager,
    ) -> anyhow::Result<Handle<T>> {
        let path = self.get_relative_path(path);

        let loader = self.get_loader::<T>(&path)?;

        log::info!("Trying to load: {}", path.display());

        let handle =
            self.load_with_loader::<T>(Source::FileSystem(path), Some(settings), loader, ass_man)?;

        Ok(handle)
    }
    pub fn load_with_data<T: Asset>(
        &self,
        path: &Path,
        data: Vec<u8>,
        settings: T::Settings,
        ass_man: AssetManager,
    ) -> anyhow::Result<Handle<T>> {
        let path = self.get_relative_path(path);

        let loader = self.get_loader::<T>(&path)?;

        log::info!(
            "Trying to load data using apparent path: {}",
            path.display()
        );

        let handle =
            self.load_with_loader::<T>(Source::Data(path, data), Some(settings), loader, ass_man)?;

        Ok(handle)
    }
    fn load_update<T: Asset>(&self, pool: &mut AssetPool<T>) -> anyhow::Result<()> {
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
                    log::info!("Loaded {:#?}", meta.source);
                    self.db.write().set_status(meta.uuid, LoadStatus::Loaded);
                    pool.insert(result.handle.index(), asset);

                    let save_path = &self.asset_dir.join(&meta.source);
                    meta.save(save_path)?;
                }
                Err(err) => {
                    log::error!("Failed to load asset {:#?}: {}", meta.source, err);
                    self.db.write().set_status(meta.uuid, LoadStatus::Failed);
                }
            }
        }

        Ok(())
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
    pub fn add_saver<T: Asset, S: Saver>(&mut self, saver: S) {
        let savers = self
            .savers
            .get_mut(&TypeId::of::<T>())
            .expect("Asset type not registered");
        savers.push(Arc::new(saver));
    }
    pub fn create<T: Asset>(
        &self,
        save_path: &Path,
        asset: T,
        pool: &mut AssetPool<T>,
    ) -> anyhow::Result<Handle<T>> {
        let asset_path = self.get_relative_path(save_path);
        let asset_path_abs = self.asset_dir.join(&asset_path);

        if asset_path_abs.exists() {
            return Err(anyhow::anyhow!(
                "Cannot create asset save path already exists"
            ));
        }
        let handle = pool.add(asset);

        let meta = MetaData::<T>::for_file(&asset_path_abs).unwrap_or_else(|| MetaData::<T> {
            source: asset_path.to_owned(),
            is_standalone: true,
            uuid: Uuid::new_v4(),
            settings: T::Settings::default(),
        });

        meta.save(&asset_path_abs)?;
        self.db.write().create_with_status(
            meta.uuid,
            asset_path.to_owned(),
            handle.clone_erased(),
            LoadStatus::Loaded,
        );

        Ok(handle)
    }
    pub fn save<T: Asset>(
        &self,
        handle: &Handle<T>,
        asset_pool: &AssetPool<T>,
    ) -> anyhow::Result<()> {
        let db = self.db.read();
        let path = db.get_path(&handle.clone_erased());
        let path = self.asset_dir.join(path);
        let saver = self.get_saver::<T>(
            path.extension()
                .expect("No extension! Cannot guess file type for saving"),
        )?;

        let asset = asset_pool
            .get(handle)
            .expect("Cannot save! Asset doesn't exist");
        let mut context = SaveContext::new(asset);

        let temp_file = tempfile::NamedTempFile::new_in(std::env::current_dir()?)?;
        let mut writer = BufWriter::new(temp_file);

        saver.save(&mut context, &mut writer)?;

        writer.into_inner()?.persist(path)?;

        Ok(())
    }
    pub fn update<T: Asset>(&self, pool: &mut AssetPool<T>) -> anyhow::Result<()> {
        self.load_update(pool)?;
        //self.save_update(pool)?;

        Ok(())
    }
    pub fn load_status(&self, handle: &ErasedHandle) -> Option<LoadStatus> {
        self.db.read().get_status(handle)
    }
    pub fn get_uuid<T: Asset>(&self, handle: &Handle<T>) -> Uuid {
        self.db.read().get_uuid(&handle.clone_erased_as_internal())
    }
    pub fn get_path<T: Asset>(&self, handle: &Handle<T>) -> PathBuf {
        self.db.read().get_path(&handle.clone_erased_as_internal())
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
}
impl AssetManager {
    pub fn set_asset_dir(&mut self, path: impl AsRef<Path>) {
        self.inner.write().set_asset_dir(path)
    }
    pub fn register_asset<T: Asset>(&mut self, pool: &AssetPool<T>) {
        self.inner.write().register_asset(pool)
    }
    pub fn add_loader<T: Asset, L: Loader>(&mut self, loader: L) {
        self.inner.write().add_loader::<T, L>(loader)
    }
    pub fn add_saver<T: Asset, S: Saver>(&mut self, saver: S) {
        self.inner.write().add_saver::<T, S>(saver)
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
    pub fn save<T: Asset>(&self, handle: &Handle<T>, pool: &AssetPool<T>) -> anyhow::Result<()> {
        self.inner.read().save(handle, pool)
    }
    pub fn create<T: Asset>(
        &self,
        save_path: &Path,
        asset: T,
        pool: &mut AssetPool<T>,
    ) -> anyhow::Result<Handle<T>> {
        self.inner.read().create(save_path, asset, pool)
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
fn txt_save_and_load() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new().init().unwrap();

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
    use rayon::ThreadPoolBuilder;
    let tp = Arc::new(ThreadPoolBuilder::new().build()?);
    let mut pool = AssetPool::default();
    let mut ass_man = AssetManager::with_threadpool(tp);
    ass_man.register_asset::<TxtFile>(&pool);
    ass_man.add_loader::<TxtFile, TxtSaverLoader>(TxtSaverLoader);
    ass_man.add_saver::<TxtFile, TxtSaverLoader>(TxtSaverLoader);

    let test = ass_man.load::<TxtFile>(Path::new("test.txt"))?;
    let test1 = ass_man.load::<TxtFile>(Path::new("DoesNotExist.txt"))?;

    assert!(ass_man.load_status(&test.clone_erased()) == Some(LoadStatus::Loading));
    assert!(ass_man.load_status(&test1.clone_erased()) == Some(LoadStatus::Loading));

    let test_erased = test.clone_erased();
    while ass_man.load_status(&test_erased) != Some(LoadStatus::Loaded) {
        ass_man.update::<TxtFile>(&mut pool)?;
    }

    let text = pool.get_mut(&test).unwrap();
    text.contents.push_str("s");

    assert!(ass_man.load_status(&test.clone_erased()) == Some(LoadStatus::Loaded));
    assert!(ass_man.load_status(&test1.clone_erased()) == Some(LoadStatus::Failed));

    ass_man.save(&test, &pool)?;

    ass_man.update::<TxtFile>(&mut pool)?;

    log::debug!("{:#?}", pool.get(&test));
    Ok(())
}
