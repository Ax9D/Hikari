pub mod project;
pub mod scene;
pub mod world;

use std::pin::Pin;

use hikari::systems::global::UnsafeGlobalState;
pub use project::*;
pub use scene::*;
pub use world::*;

pub type EngineState<'a> = Pin<&'a UnsafeGlobalState>;
