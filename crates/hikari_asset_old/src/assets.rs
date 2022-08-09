use std::sync::{atomic::AtomicUsize, Arc};

use flume::{Receiver, Sender};

use crate::handle::{HandleIndex, RefOp};

use super::Handle;

pub(crate) struct HandleAllocator {
    index: AtomicUsize,
    free_list: Receiver<usize>,
    free_list_sender: Sender<usize>,
    ref_send: Sender<RefOp>,
    ref_recv: Receiver<RefOp>,
}

impl HandleAllocator {
    pub(crate) fn new(ref_send: Sender<RefOp>, ref_recv: Receiver<RefOp>) -> Self {
        let (free_list_sender, free_list) = flume::unbounded();
        Self {
            index: AtomicUsize::new(0),
            free_list,
            free_list_sender,
            ref_send,
            ref_recv,
        }
    }
    pub fn allocate<T>(&self) -> Handle<T> {
        let index = self.free_list.try_recv().unwrap_or_else(|_| {
            self.index
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        });

        let sender = self.ref_send.clone();

        Handle::new(index, sender)
    }
    pub(crate) fn ref_op_channel(&self) -> &Receiver<RefOp> {
        &self.ref_recv
    }
    ///Assumes that there are no references to the current handle
    pub fn deallocate(&self, handle_index: HandleIndex) {
        self.free_list_sender
            .send(handle_index)
            .expect("Failed to deallocate handle");
    }
}

pub struct Assets<T> {
    pool: Vec<Option<T>>,
    handle_allocator: Arc<HandleAllocator>,
}

impl<T> Assets<T> {
    pub fn new() -> Self {
        let (ref_send, ref_recv) = flume::unbounded();
        Self {
            pool: Vec::new(),
            handle_allocator: Arc::new(HandleAllocator::new(ref_send, ref_recv)),
        }
    }
    pub(crate) fn handle_allocator(&self) -> &Arc<HandleAllocator> {
        &self.handle_allocator
    }
    fn allocate_handle(&self) -> Handle<T> {
        self.handle_allocator.allocate()
    }
    fn ensure_length(&mut self, ix: usize) {
        if ix >= self.pool.len() {
            self.pool.resize_with(ix + 1, || None);
        }
    }
    pub fn push(&mut self, asset: T) -> Handle<T> {
        let handle = self.handle_allocator.allocate();
        self.insert(handle.index(), asset);

        handle
    }
    pub(crate) fn remove(&mut self, index: HandleIndex) -> Option<T> {
        self.pool[index].take()
    }
    pub(crate) fn insert(&mut self, index: HandleIndex, asset: T) {
        self.ensure_length(index);

        self.pool[index] = Some(asset);
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
#[cfg(test)]
mod test {}
