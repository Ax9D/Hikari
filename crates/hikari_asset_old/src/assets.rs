use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};

use crate::handle::{Handle, HandleAllocator, RefOp};
#[allow(unused)]
struct RefCounts {
    ref_send: flume::Sender<RefOp>,
    ref_recv: flume::Receiver<RefOp>,
}
impl RefCounts {
    pub fn new() -> Self {
        let (ref_send, ref_recv) = flume::unbounded();
        Self { ref_send, ref_recv }
    }
}
#[allow(unused)]
pub struct AssetPool<T> {
    pool: Vec<Option<T>>,
    handle_allocator: Arc<HandleAllocator>,
    ref_counts: RefCounts,
}
impl<T> Default for AssetPool<T> {
    fn default() -> Self {
        let ref_counts = RefCounts::new();
        Self {
            pool: Default::default(),
            handle_allocator: Arc::new(HandleAllocator::new(ref_counts.ref_send.clone())),
            ref_counts,
        }
    }
}
impl<T: 'static> AssetPool<T> {
    fn ensure_length(&mut self, ix: usize) {
        if ix >= self.pool.len() {
            self.pool.resize_with(ix + 1, || None);
        }
    }
    pub(crate) fn handle_allocator(&self) -> &Arc<HandleAllocator> {
        &self.handle_allocator
    }
    pub fn insert(&mut self, handle_ix: usize, data: T) {
        self.ensure_length(handle_ix);
        self.pool[handle_ix] = Some(data);
    }
    pub fn add(&mut self, data: T) -> Handle<T> {
        let handle = self.handle_allocator.allocate();
        let index = handle.index();
        self.ensure_length(index);
        self.pool[index] = Some(data);

        handle
    }
    pub fn remove(&mut self, handle_ix: usize) -> Option<T> {
        self.handle_allocator.deallocate(handle_ix);
        self.pool[handle_ix].take()
    }
    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        match self.pool.get(handle.index()) {
            Some(data) => data.as_ref(),
            None => None,
        }
    }
    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        match self.pool.get_mut(handle.index()) {
            Some(data) => data.as_mut(),
            None => None,
        }
    }
}

#[derive(Default)]
pub struct AssetStorage {
    map: HashMap<TypeId, Mutex<Box<dyn Any + Send + Sync + 'static>>>,
}

impl AssetStorage {
    pub fn add<T: Send + Sync + 'static>(&mut self) {
        let previous = self.map.insert(
            TypeId::of::<T>(),
            Mutex::new(Box::new(AssetPool::<T>::default())),
        );
        if previous.is_some() {
            panic!("AssetPool of type: {} already exists!", type_name::<T>());
        }
    }
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<MappedMutexGuard<AssetPool<T>>> {
        let any_pool = self.map.get(&TypeId::of::<T>())?;

        let guard: MappedMutexGuard<AssetPool<T>> = MutexGuard::map(any_pool.lock(), |any_pool| {
            any_pool
                .downcast_mut::<AssetPool<T>>()
                .expect("AssetPool type mismatch")
        });

        Some(guard)
    }
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut AssetPool<T>> {
        let any_pool = self.map.get_mut(&TypeId::of::<T>())?;

        let any_pool = any_pool.get_mut();
        Some(
            any_pool
                .downcast_mut::<AssetPool<T>>()
                .expect("AssetPool type mismatch"),
        )
    }
}

#[test]
fn add_pool() {
    use crate::Asset;
    struct Texture;
    impl Asset for Texture {
        type Settings = ();
    }
    let mut storage = AssetStorage::default();
    storage.add::<Texture>();
}
#[test]
fn get_pool() {
    use crate::Asset;
    struct Texture;
    impl Asset for Texture {
        type Settings = ();
    }
    let mut storage = AssetStorage::default();
    storage.add::<Texture>();
    assert!(storage.get::<Texture>().is_some());
    assert!(storage.get_mut::<Texture>().is_some());
}
