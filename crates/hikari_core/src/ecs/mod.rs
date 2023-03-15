pub mod registry;
pub mod world;

pub use registry::*;
pub use world::*;

#[cfg(feature = "serde")]
pub mod serialize;

pub trait Component: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Component for T {}
