pub mod world;
pub mod registry;

pub use world::*;
pub use registry::*;

#[cfg(feature = "serde")]
pub mod serialize;

pub trait Component: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Component for T {}
