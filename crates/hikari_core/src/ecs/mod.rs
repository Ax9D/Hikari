pub mod scene;

pub use scene::*;

#[cfg(feature = "serde")]
pub mod serde;

pub trait Component: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Component for T {}
