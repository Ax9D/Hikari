use std::{any::TypeId, collections::HashMap};
use crate::ErasedHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadStatus {
    Loaded,
    Loading,
    Unloaded,
    Failed,
}

#[derive(Default)]
pub struct LoadStatuses {
    inner: HashMap<ErasedHandle, LoadStatus>,
}
impl LoadStatuses {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
    pub fn insert(&mut self, handle: &ErasedHandle, status: LoadStatus) {
        assert!(handle.is_internal());

        self.inner.insert(handle.clone(), status);
    }

    pub fn get(&self, handle: &ErasedHandle) -> Option<LoadStatus> {
        self.inner.get(handle).copied()
    }
    pub fn get_mut(&mut self, handle: &ErasedHandle) -> Option<&mut LoadStatus> {

        self.inner.get_mut(handle)
    }
    pub fn remove(&mut self, handle: &ErasedHandle) -> Option<LoadStatus> {
        assert!(handle.is_internal());

        self.inner.remove(handle)
    }
    pub fn remove_by_ix(&mut self, handle: &(TypeId, usize)) -> Option<LoadStatus> {
        let mut removed = None;
        self.inner = self.inner.drain().filter(|(current, status)| {
            let remove = current.type_id_asset() == handle.0 && current.index() == handle.1;

            if remove {
                removed = Some(*status);
            }

            !remove
        }).collect();

        removed
    }
    // pub fn get_mut(&self, handle: &InternalHandle) -> MappedRwLockWriteGuard<'_, LoadStatus> {
    //     RwLockWriteGuard::map(self.inner.write(), |x| x.get_mut(handle).unwrap())
    // }
    // pub fn full_lock(&self) -> RwLockWriteGuard<'_, HashMap<InternalHandle, LoadStatus>> {
    //     self.inner.write()
    // }
}

