use fxhash::{FxBuildHasher, FxHashMap};
use lru::LruCache;
use std::{error::Error, hash::Hash, mem::ManuallyDrop};

pub struct CacheMap<K, V> {
    cache: LruCache<K, V, FxBuildHasher>,

    unused: Vec<V>,
}

impl<K: Hash + Eq + Clone, V: Copy> CacheMap<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::with_hasher(capacity, FxBuildHasher::default()),
            unused: Vec::new(),
        }
    }
    unsafe fn unsafe_copy<T>(src: &T) -> T {
        let mut dst = std::mem::zeroed();
        std::ptr::copy_nonoverlapping(src as *const T, &mut dst as *mut T, 1);
        dst
    }
    pub fn get<'a, E: Error>(&'a mut self, key: &K, build_fn: impl FnOnce(&K) -> Result<V, E>) ->  Result<&'a V, E> {
        let value;
        match self.cache.get(key) {
            Some(value) => Ok( value ),

            None => {
                match self.cache.peek_lru() {
                    Some((_, value)) => {
                        self.unused.push( unsafe { Self::unsafe_copy(value) } );
                    },
                    None => {},
                }

                let new_value = (build_fn)(key)?;
                self.cache.put(key.clone(), new_value);

                value = self.cache.get(key);
                Ok( value.as_deref().unwrap() )
            }
        }
    }
    pub fn unused(&mut self) -> &mut Vec<V> {
        &mut self.unused
    }
}