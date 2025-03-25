use std::sync::atomic::AtomicUsize;

use crate::Asset;

use hikari_handle::RefMessage;
use super::typed::Handle;

pub(crate) struct HandleAllocator {
    handle_count: AtomicUsize,
    free_list_recv: flume::Receiver<usize>,

    free_list_send: flume::Sender<usize>,
    refcount_send: flume::Sender<RefMessage>,
}
impl HandleAllocator {
    pub fn new(refcount_send: flume::Sender<RefMessage>) -> Self {
        let (free_list_send, free_list_recv) = flume::unbounded();
        Self {
            handle_count: AtomicUsize::new(0),
            free_list_recv,
            free_list_send,
            refcount_send,
        }
    }
    pub fn allocate<T: Asset>(&self) -> Handle<T> {
        let index = self.allocate_index();

        Handle::new(index, self.refcount_send.clone())
    }
    #[allow(unused)]
    pub fn refcount_sender(&self) -> &flume::Sender<RefMessage> {
        &self.refcount_send
    }
    pub fn allocate_index(&self) -> usize {
        let index = self.free_list_recv.try_recv().unwrap_or_else(|_| {
            self.handle_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        });

        index
    }
    pub fn deallocate_index(&self, index: usize) {
        self.free_list_send
            .send(index)
            .expect("Failed to update free list");
    }
}


// pub struct RawHandle {
//     index: usize,
//     ref_counter: RefCounter
// }

// impl PartialEq for RawHandle {
//     fn eq(&self, other: &Self) -> bool {
//         self.index == other.index
//     }
// }
// impl Eq for RawHandle {}

// impl std::hash::Hash for RawHandle {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.index.hash(state);
//     }
// }

// impl RawHandle {
//     pub(crate) fn new(index: usize, ref_send: flume::Sender<RefMessage>) -> Self {
//         Self {
//             index,
//             ref_counter: RefCounter::new(index, ref_send),
//         }
//     }
//     pub fn index(&self) -> usize {
//         self.index
//     }
//     pub fn clone_weak(&self) -> Self {
//         Self {
//             index: self.index,
//             ref_counter: self.ref_counter.clone_weak()
//         }
//     }
//     pub fn to_weak(self) -> Self {
//         Self {
//             index: self.index,
//             ref_counter: self.ref_counter.to_weak()
//         }
//     }
//     pub fn make_strong(&mut self, ref_send: &flume::Sender<usize>) -> Self {
//         todo!();//Self { index: self.index, ref_counter: self.ref_counter.make_strong(self.index, ref_send) }
//     }
//     pub fn strong_count(&self) -> usize {
//         self.ref_counter.strong_count()
//     }
//     pub fn weak_count(&self) -> usize {
//         self.ref_counter.weak_count()
//     }
//     pub fn is_weak(&self) -> bool {
//         self.ref_counter.is_weak()
//     }
// }

// impl Clone for RawHandle {
//     fn clone(&self) -> Self {
//         Self {
//             index: self.index,
//             ref_counter: self.ref_counter.clone()
//         }
//     }
// }
