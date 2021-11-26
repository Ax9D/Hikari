#![allow(dead_code)]
mod sync;
mod unsync;


#[cfg(feature = "thread_unsafety")]
pub use unsync::*;

#[cfg(not(feature = "thread_unsafety"))]
pub use sync::*;