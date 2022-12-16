use std::hash::Hash;
use std::{any::TypeId, marker::PhantomData, sync::atomic::AtomicUsize};

use crate::Asset;

pub struct RawHandle {
    index: usize,
    kind: HandleKind,
}
#[allow(unused)]
pub(crate) enum RefOp {
    Increment(usize),
    Decrement(usize),
}
#[allow(unused)]
#[derive(Clone)]
pub(crate) enum HandleKind {
    Strong(flume::Sender<RefOp>),
    Weak(flume::Sender<RefOp>),
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
    pub(crate) fn new(index: usize, ref_send: flume::Sender<RefOp>) -> Self {
        Self {
            index,
            kind: HandleKind::Strong(ref_send),
        }
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn clone_weak(&self) -> Self {
        match self.kind.clone() {
            HandleKind::Strong(chan) | HandleKind::Weak(chan) => Self {
                kind: HandleKind::Weak(chan),
                index: self.index,
            },
        }
    }
    pub fn clone_strong(&self) -> Self {
        match self.kind.clone() {
            HandleKind::Strong(chan) | HandleKind::Weak(chan) => Self {
                kind: HandleKind::Strong(chan),
                index: self.index,
            },
        }
    }
}

impl Clone for RawHandle {
    fn clone(&self) -> Self {
        // self.ref_send
        //     .send(RefOp::Increment(self.index))
        //     .expect("Failed to increment reference count");
        Self {
            index: self.index.clone(),
            kind: self.kind.clone(),
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
    pub fn raw(&self) -> &RawHandle {
        &self.raw
    }
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
    pub fn clone_strong(&self) -> Self {
        Self {
            raw: self.raw.clone_strong(),
            type_id: self.type_id,
        }
    }
    pub fn is_weak(&self) -> bool {
        matches!(self.raw.kind, HandleKind::Weak(_))
    }
}
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

impl<T: 'static> Handle<T> {
    pub(crate) fn new(index: usize, ref_send: flume::Sender<RefOp>) -> Self {
        Self {
            raw: RawHandle::new(index, ref_send),
            _phantom: PhantomData,
        }
    }
    pub fn index(&self) -> usize {
        self.raw.index()
    }
    pub fn clone_erased(&self) -> ErasedHandle {
        self.clone().into()
    }
    pub fn clone_erased_as_weak(&self) -> ErasedHandle {
        ErasedHandle {
            raw: self.raw.clone_weak(),
            type_id: TypeId::of::<T>(),
        }
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
    #[allow(unused)]
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
    pub fn allocate<T: Asset>(&self) -> Handle<T> {
        let index = self.free_list_recv.try_recv().unwrap_or_else(|_| {
            self.handle_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        });

        Handle::new(index, self.refcount_send.clone())
    }
    #[allow(unused)]
    pub fn deallocate<T: Asset>(&self, handle: Handle<T>) {
        self.free_list_send
            .send(handle.index())
            .expect("Failed to update free list");
    }
}
