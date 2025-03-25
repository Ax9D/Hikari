use std::marker::PhantomData;

use crate::Asset;

use hikari_handle::RawHandle;
use hikari_handle::RefMessage;
use super::erased::ErasedHandle;

pub struct Handle<T> {
    raw: RawHandle,
    _phantom: PhantomData<T>,
}
impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}
impl<T> Eq for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}

impl<T> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<T: Asset> Handle<T> {
    pub(crate) fn new(index: usize, ref_send: flume::Sender<RefMessage>) -> Self {
        Self {
            raw: RawHandle::strong(index, ref_send),
            _phantom: PhantomData,
        }
    }
    pub(crate) fn from_raw(raw: RawHandle) -> Self {
        Self {
            raw,
            _phantom: PhantomData
        }
    }
    pub fn index(&self) -> usize {
        self.raw.index()
    }
    pub fn clone_erased(&self) -> ErasedHandle {
        self.clone().into()
    }
    pub fn clone_erased_as_weak(&self) -> ErasedHandle {
        ErasedHandle::from_raw::<T>(self.raw.downgrade_weak())
    }
    pub fn clone_erased_as_internal(&self) -> ErasedHandle {
        ErasedHandle::from_raw::<T>(self.raw.downgrade_internal())
    }
    pub fn upgrade_strong(&self) -> Option<Self> {
        Some(Self {
            raw: self.raw.upgrade_strong()?,
            _phantom: PhantomData
        })
    }
    pub fn to_weak(self) -> Self {
        Self {
            raw: self.raw.clone().downgrade_weak(),
            _phantom: PhantomData
        }
    }
    pub fn strong_count(&self) -> usize {
        self.raw.strong_count()
    }
    pub fn weak_count(&self) -> usize {
        self.raw.weak_count()
    }
    pub fn is_weak(&self) -> bool {
        self.raw.is_weak()
    }
}
impl<T> Drop for Handle<T> {
    fn drop(&mut self) {
        if !self.raw.is_weak() {
            // log::debug!("Dropping handle {}[{}] S: {} W: {} Internal: {}", std::any::type_name::<T>(), self.raw.index(), self.raw.strong_count() - 1, self.raw.weak_count(), self.raw.is_internal());
        }
        // self.ref_send
        //     .send(RefOp::Decrement(self.index))
        //     .expect("Failed to decrement reference count");
    }
}
impl<T: Asset> Into<ErasedHandle> for Handle<T> {
    fn into(self) -> ErasedHandle {
        ErasedHandle::from_raw::<T>(self.raw.clone())
    }
}