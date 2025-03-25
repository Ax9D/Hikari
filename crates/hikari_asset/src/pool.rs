use std::any::{Any, TypeId};

use crate::{Asset, ErasedHandle, Handle, HandleAllocator, RefMessage};
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

pub type PoolRef<'a, T> = MappedRwLockReadGuard<'a, AssetPool<T>>;
pub type PoolMut<'a, T> = MappedRwLockWriteGuard<'a, AssetPool<T>>;

pub struct AssetPool<T> {
    inner: Vec<Option<T>>,
    handle_allocator: HandleAllocator,
    ref_count_recv: flume::Receiver<RefMessage>,
}
impl<T: Asset> AssetPool<T> {
    pub fn new() -> Self {
        let (sender, recv) = flume::unbounded();
        Self {
            inner: Vec::new(),
            handle_allocator: HandleAllocator::new(sender),
            ref_count_recv: recv
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
    pub(crate) fn insert_with_handle(&mut self, handle_index: usize, data: T) {
        self.ensure_length(handle_index);

        self.inner[handle_index] = Some(data);
    }
    pub fn insert(&mut self, data: T) -> Handle<T> {
        let handle = self.handle_allocator.allocate::<T>();
        self.insert_with_handle(handle.index(), data);

        handle
    }
    pub fn garbage_collect(&mut self, mut dealloc_cbk: impl FnMut(usize)) {
        for message in self.ref_count_recv.try_iter() {
            match message {
                RefMessage::Unload(index) => {
                    self.inner[index].take();

                    log::debug!("Unload: {}[{}]", std::any::type_name::<T>(), index);
                },
                RefMessage::Deallocate(index) => {
                    self.handle_allocator().deallocate_index(index);
                    assert!(self.inner[index].is_none());
                    (dealloc_cbk)(index);

                    log::debug!("Deallocate handle {}[{}]", std::any::type_name::<T>(), index);
                },
            }
        }
    }
    pub(crate) fn get_by_index(&self, index: usize) -> Option<&T> {
        match self.inner.get(index) {
            Some(asset) => asset.as_ref(),
            None => None,
        }
    }
    pub(crate) fn get_mut_by_index(&mut self, index: usize) -> Option<&mut T> {
        match self.inner.get_mut(index) {
            Some(asset) => asset.as_mut(),
            None => None,
        }
    }
    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        self.get_by_index(handle.index())
    }
    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.get_mut_by_index(handle.index())
    }
    /// Takes the asset from the pool
    /// Returns `None` if the asset is either not loaded or if there is more than 1 strong reference to asset 
    pub fn take(&mut self, handle: &Handle<T>) -> Option<T> {
        match self.inner.get_mut(handle.index()) {
            Some(asset) => {
                if handle.strong_count() == 1 {
                    asset.take()
                } else {
                    None
                }
            },
            None => None
        }
    }
    pub fn get_erased(&self, handle: &ErasedHandle) -> Option<&T> {
        assert!(TypeId::of::<T>() == handle.type_id_asset());
        self.get_by_index(handle.index())
    }
    pub fn get_mut_erased(&mut self, handle: &ErasedHandle) -> Option<&mut T> {
        assert!(TypeId::of::<T>() == handle.type_id_asset());
        self.get_mut_by_index(handle.index())
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
