use std::sync::atomic::{AtomicUsize, Ordering};

use ash::vk;

struct IndexAllocator {
    new_index: AtomicUsize,
    freed_indices: flume::Receiver<usize>,
    freed_indices_sender: flume::Sender<usize>,
}

impl IndexAllocator {
    pub fn allocate(&self) -> usize {
        self.freed_indices
            .try_iter()
            .next()
            .unwrap_or_else(|| self.new_index.fetch_add(1, Ordering::Relaxed))
    }
    pub fn deallocate(&self, index: usize) {
        self.freed_indices_sender
            .send(index)
            .expect("Failed to send")
    }
}

pub struct BindlessSet {
    textures: BindlessTextures,
}

struct RegisterTexture {
    view: vk::ImageView,
    sampler: vk::Sampler,
    layout: vk::ImageLayout,
}
struct BindlesTexturesInner {
    index_allocator: IndexAllocator,
}
impl BindlesTexturesInner {
    // pub fn register_texture(&self, view: vk::ImageView, sampler: vk::Sampler, layout: vk::ImageLayout) -> usize {
    //     let index = self.index_allocator.allocate();

    //     self.register_request_sender.send(
    //         RegisterTexture {
    //                 view,
    //              sampler,
    //              layout
    //             }
    //     ).expect("Failed to send Register Request");

    //     index
    // }
    // pub fn deregister_texture(&self, index: usize) {
    //     self.index_allocator.deallocate(index)
    // }
    // pub fn next_frame(&mut self) {
    //     self.register_request_recv
    // }
}

pub struct BindlessTextures {}

impl BindlessTextures {}
