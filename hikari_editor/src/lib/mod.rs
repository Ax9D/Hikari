pub mod project;
use std::pin::Pin;

use hikari::core::global::UnsafeGlobalState;
pub use project::*;

pub type EngineState<'a> = Pin<&'a UnsafeGlobalState>;
