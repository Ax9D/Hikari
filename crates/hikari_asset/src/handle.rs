use std::hash::Hash;
use std::{any::TypeId, marker::PhantomData, sync::atomic::AtomicUsize};

use crate::Asset;

pub(crate) enum RefOp {
    Increment(usize),
    Decrement(usize),
}
struct RawHandle {
    index: usize,
    ref_send: flume::Sender<RefOp>,
}

impl PartialEq for RawHandle {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl Eq for RawHandle {}

impl Hash for RawHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl RawHandle {
    pub fn new(index: usize, ref_send: flume::Sender<RefOp>) -> Self {
        Self { index, ref_send }
    }
}

impl Clone for RawHandle {
    fn clone(&self) -> Self {
        // self.ref_send
        //     .send(RefOp::Increment(self.index))
        //     .expect("Failed to increment reference count");
        Self {
            index: self.index.clone(),
            ref_send: self.ref_send.clone(),
        }
    }
}
impl Drop for RawHandle {
    fn drop(&mut self) {
        // self.ref_send
        //     .send(RefOp::Decrement(self.index))
        //     .expect("Failed to decrement reference count");
    }
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ErasedHandle {
    raw: RawHandle,
    type_id: TypeId,
}

impl ErasedHandle {
    pub fn into_typed<T: Asset>(self) -> Option<Handle<T>> {
        if TypeId::of::<T>() == self.type_id {
            Some(Handle {
                raw: self.raw,
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }
    pub fn clone_typed<T: Asset>(&self) -> Option<Handle<T>> {
        self.clone().into_typed::<T>()
    }
}
#[derive(PartialEq, Eq)]
pub struct Handle<T> {
    raw: RawHandle,
    _phantom: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}

impl<T: 'static> Handle<T> {
    pub(crate) fn new(index: usize, ref_send: flume::Sender<RefOp>) -> Self {
        Self {
            raw: RawHandle::new(index, ref_send),
            _phantom: PhantomData,
        }
    }
    pub(crate) fn index(&self) -> usize {
        self.raw.index
    }
    pub fn clone_erased(&self) -> ErasedHandle {
        self.clone().into()
    }
}

impl<T: 'static> Into<ErasedHandle> for Handle<T> {
    fn into(self) -> ErasedHandle {
        ErasedHandle {
            raw: self.raw,
            type_id: TypeId::of::<T>(),
        }
    }
}

pub(crate) struct HandleAllocator {
    handle_count: AtomicUsize,
    free_list_recv: flume::Receiver<usize>,
    free_list_send: flume::Sender<usize>,
    refcount_send: flume::Sender<RefOp>,
}
impl HandleAllocator {
    pub fn new(refcount_send: flume::Sender<RefOp>) -> Self {
        let (free_list_send, free_list_recv) = flume::unbounded();
        Self {
            handle_count: AtomicUsize::new(0),
            free_list_recv,
            free_list_send,
            refcount_send,
        }
    }
    pub fn allocate<T: 'static>(&self) -> Handle<T> {
        let index = self.free_list_recv.try_recv().unwrap_or_else(|_| {
            self.handle_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        });

        Handle::new(index, self.refcount_send.clone())
    }
    pub fn deallocate(&self, index: usize) {
        self.free_list_send
            .send(index)
            .expect("Failed to update free list");
    }
}
