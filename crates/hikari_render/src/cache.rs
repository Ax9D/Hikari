use std::collections::HashMap;
use parking_lot::{Mutex, MutexGuard};

use crate::{descriptor::DescriptorSetLayoutCache, image::SamplerCache};

type Map<K, V> = HashMap<K, V, hikari_utils::hash::BuildHasher>;

pub(crate) struct ResourceCache {
    sampler_cache: Mutex<SamplerCache>,
    set_layout_cache: Mutex<DescriptorSetLayoutCache>
}

impl ResourceCache {
    pub fn new(device: &ash::Device) -> Self {
        Self {
            sampler_cache: Mutex::new(SamplerCache::new(device)),
            set_layout_cache: Mutex::new(DescriptorSetLayoutCache::new(device))
        }
    }
    pub fn sampler(&self) -> MutexGuard<'_, SamplerCache> {
        self.sampler_cache.lock()
    }
    pub fn set_layout(&self) -> MutexGuard<'_, DescriptorSetLayoutCache> {
        self.set_layout_cache.lock()
    }
}

