use std::{sync::Arc, path::{Path}};

use rayon::ThreadPool;

mod assets;
mod manager;
mod handle;
mod meta;

pub use assets::*;
pub use manager::*;
pub use handle::*;

pub trait Asset: Send + Sync + 'static {
    const NAME: &'static str;
}
pub trait Load: Send + Sync + Sized + 'static {
    type Loader: Send + Sync + 'static;
    type LoadSettings: Default;
    fn load(
        bytes: &[u8],
        meta: &LoadMetadata,
        settings: &Self::LoadSettings,
        loader: &Self::Loader,
    ) -> Result<Self, anyhow::Error>;
}

pub struct LoadMetadata<'path> {
    pub path: &'path Path
}

struct AssetManagerPlugin;

impl hikari_core::Plugin for AssetManagerPlugin {
    fn build(self, game: &mut hikari_core::Game) {
        let threadpool = game.get::<Arc<ThreadPool>>().clone();
        game.add_state(AssetManager::new(&threadpool));
    }
}