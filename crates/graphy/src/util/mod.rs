mod cache_map;
mod temporary_map;
mod intrusive_linked_list;

use std::ops::Range;

pub use cache_map::CacheMap;
pub use temporary_map::TemporaryMap;

pub fn for_each_bit(mask: u32, range: Range<usize>,  mut f: impl FnMut(u32)) {
    for i in range {
        let value = (mask >> i) & 1;

        if value == 1 {
            (f)(value);
        }
    }
}