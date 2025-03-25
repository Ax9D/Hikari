use fxhash;

pub type BuildHasher = fxhash::FxBuildHasher;

#[inline]
pub fn hasher_builder() -> BuildHasher {
    fxhash::FxBuildHasher::default()
}

pub type Hasher = fxhash::FxHasher;

#[inline]
pub fn hasher() -> Hasher {
    fxhash::FxHasher::default()
}

use std::{hash::Hasher as OtherHasher, collections::HashMap};

pub fn quick_hash(data: impl std::hash::Hash) -> u64 {
    let mut state = hasher();

    data.hash(&mut state);

    state.finish()
}

pub type BuildNoHashHasher<T> = nohash_hasher::BuildNoHashHasher<T>;
pub type NoHashHasher<T> = nohash_hasher::NoHashHasher<T>;
pub use nohash_hasher::IsEnabled;

pub type NoHashMap<K, V> = HashMap<K, V, BuildNoHashHasher<K>>;