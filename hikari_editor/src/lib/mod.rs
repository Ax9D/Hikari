pub mod project;
pub mod scene;

use std::pin::Pin;

use hikari::systems::global::UnsafeGlobalState;
pub use project::*;
pub use scene::*;

pub type EngineState<'a> = Pin<&'a UnsafeGlobalState>;
