pub mod scene;

pub use scene::*;

#[cfg(feature = "serialize")]
pub mod serialize;

pub trait Component: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Component for T {}