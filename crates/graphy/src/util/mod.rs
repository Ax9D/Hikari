mod arrayvec_copy;
mod cache_map;
mod intrusive_linked_list;
mod perframe;
mod temporary_map;

use std::ops::Range;

pub use arrayvec_copy::ArrayVecCopy;
pub use cache_map::CacheMap;
pub use perframe::PerFrame;
pub use temporary_map::TemporaryMap;

#[inline]
pub fn for_each_bit_in_range(mask: u32, range: Range<usize>, mut f: impl FnMut(u32)) {
    for i in range {
        let value = (mask >> i) & 1;

        if value == 1 {
            (f)(i as u32);
        }
    }
}

#[inline]
pub fn for_each_bit(mut mask: u32, mut f: impl FnMut(u32)) {
    let mut ix = 0;
    while mask != 0 {
        let value = mask & 1;

        if value == 1 {
            (f)(ix);
        }
        mask >>= 1;
        ix += 1;
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

use std::hash::Hasher as OtherHasher;

pub fn quick_hash(data: impl std::hash::Hash) -> u64 {
    let mut state = hasher();

    data.hash(&mut state);

    state.finish()
}
