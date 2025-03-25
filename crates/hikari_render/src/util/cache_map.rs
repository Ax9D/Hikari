use lru::LruCache;
use std::{hash::Hash};

pub struct CacheMap<K, V> {
    cache: LruCache<K, V, crate::util::BuildHasher>,

    unused: Vec<V>,
}

impl<K: Hash + Eq + Clone, V: Copy> CacheMap<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::with_hasher(capacity, crate::util::hasher_builder()),
            unused: Vec::new(),
        }
    }
    unsafe fn unsafe_copy<T>(src: &T) -> T {
        let mut dst = std::mem::zeroed();
        std::ptr::copy_nonoverlapping(src as *const T, &mut dst as *mut T, 1);
        dst
    }
    pub fn get<'a, 'b, E>(
        &'a mut self,
        key: &'b K,
        build_fn: impl FnOnce(&K) -> Result<V, E>,
    ) -> Result<&'a V, E> {
        hikari_dev::profile_function!();
        
        let value;

        if self.cache.get(key).is_none() {
            let will_evict = self.cache.len() == self.cache.cap();

            if will_evict {
                if let Some((_, value)) = self.cache.peek_lru() {
                    self.unused.push(unsafe { Self::unsafe_copy(value) });
                }
            }

            let new_value = (build_fn)(key)?;
            self.cache.put(key.clone(), new_value);

            value = self.cache.get(key);
            Ok(value.unwrap())
        } else {
            Ok(self.cache.get(key).unwrap())
        }
    }
    pub fn unused(&mut self) -> &mut Vec<V> {
        &mut self.unused
    }
}
