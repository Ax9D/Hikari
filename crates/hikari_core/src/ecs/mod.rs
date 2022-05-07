pub mod scene;
pub use scene::*;

#[cfg(feature = "serialize")]
pub mod serialize;

pub trait Component: hecs::Component {}

impl<T: hecs::Component> Component for T {}
