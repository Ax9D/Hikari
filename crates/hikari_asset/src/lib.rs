#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_unsafe)]

//pub mod texture;
mod asset;
mod assets;
mod handle;
mod manager;
mod meta;
mod serde;

pub use crate::serde::*;
pub use asset::*;
pub use assets::*;
pub use handle::*;
pub use manager::*;
pub use meta::*;

pub type Error = anyhow::Error;
//pub use scene::Scene;
//pub use texture::Texture;