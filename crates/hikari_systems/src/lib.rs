#[allow(unused)]
pub mod global;
pub mod query;
pub mod state;
pub mod task;
pub mod system;

pub use global::GlobalState;
pub use global::GlobalStateBuilder;
pub use state::State;

mod borrow;
