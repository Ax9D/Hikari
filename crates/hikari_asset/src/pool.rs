use std::any::Any;

use crate::{Asset, Handle, HandleAllocator};
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

pub type PoolRef<'a, T> = MappedRwLockReadGuard<'a, AssetPool<T>>;
pub type PoolMut<'a, T> = MappedRwLockWriteGuard<'a, AssetPool<T>>;

pub struct AssetPool<T> {
    inner: Vec<Option<T>>,
    handle_allocator: HandleAllocator,
}
impl<T: Asset> AssetPool<T> {
    pub fn new() -> Self {
        //TODO: Implement Reference Counting
        let (sender, _recv) = flume::unbounded();
        Self {
            inner: Vec::new(),
            handle_allocator: HandleAllocator::new(sender),
        }
    }
    fn ensure_length(&mut self, ix: usize) {
        if ix >= self.inner.len() {
            self.inner.resize_with(ix + 1, || None);
        }
    }
    pub(crate) fn handle_allocator(&self) -> &HandleAllocator {
        &self.handle_allocator
    }
    pub(crate) fn insert_with_handle(&mut self, handle: &Handle<T>, data: T) {
        self.ensure_length(handle.index());

        self.inner[handle.index()] = Some(data);
    }
    pub fn insert(&mut self, data: T) -> Handle<T> {
        let handle = self.handle_allocator.allocate::<T>();
        self.insert_with_handle(&handle, data);

        handle
    }
    #[allow(unused)]
    pub(crate) fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let removed = self.inner[handle.index()].take();
        self.handle_allocator.deallocate(handle);

        removed
    }
    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        match self.inner.get(handle.index()) {
            Some(asset) => asset.as_ref(),
            None => None,
        }
    }
    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        match self.inner.get_mut(handle.index()) {
            Some(asset) => asset.as_mut(),
            None => None,
        }
    }
}

pub(crate) struct DynAssetPool {
    inner: RwLock<Box<dyn Any + Send + Sync + 'static>>,
}

impl DynAssetPool {
    pub fn new<T: Asset>() -> Self {
        Self {
            inner: RwLock::new(Box::new(AssetPool::<T>::new())),
        }
    }
    #[allow(unused)]
    pub fn get_mut<T: Asset>(&mut self) -> &mut AssetPool<T> {
        self.inner.get_mut().downcast_mut().unwrap()
    }
    pub fn read<T: Asset>(&self) -> PoolRef<T> {
        RwLockReadGuard::map(self.inner.read(), |lock| {
            lock.downcast_ref::<AssetPool<T>>().unwrap()
        })
    }
    pub fn write<T: Asset>(&self) -> PoolMut<T> {
        RwLockWriteGuard::map(self.inner.write(), |lock| {
            lock.downcast_mut::<AssetPool<T>>().unwrap()
        })
    }
}
