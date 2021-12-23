mod cache_map;
mod intrusive_linked_list;
mod temporary_map;

use std::ops::Range;

pub use cache_map::CacheMap;
pub use temporary_map::TemporaryMap;

#[inline]
pub fn for_each_bit(mut mask: u32, range: Range<usize>, mut f: impl FnMut(u32)) {
    for i in range {
        let value = (mask >> i) & 1;

        if value == 1 {
            (f)(i as u32);
        }
    }
}

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