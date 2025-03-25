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


pub type BuildHasher = hikari_utils::hash::BuildHasher;

pub use hikari_utils::hash::*;

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
    while mask != 0 {
        let t = mask & mask.wrapping_neg();
        (f)(mask.trailing_zeros());
        mask ^= t;
    }
}

pub fn n_workgroups(num_elements: u32, n_threads: u32) -> u32 {
    let mut n_workgroups = num_elements / n_threads;
    if num_elements % n_threads > 1 {
        n_workgroups += 1;
    }

    n_workgroups
}
