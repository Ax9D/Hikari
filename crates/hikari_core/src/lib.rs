mod ecs;
mod game;
mod plugin;
mod window;

use std::sync::Arc;

pub use ecs::*;
pub use game::*;
use hikari_asset::{AssetManager, AssetStorage};
pub use plugin::*;
use rayon::ThreadPoolBuilder;

pub const FIRST: &'static str = "First";
pub const UPDATE: &'static str = "Update";
pub const RENDER: &'static str = "Render";
pub const LAST: &'static str = "Last";
pub struct CorePlugin;

impl crate::Plugin for CorePlugin {
    fn build(self, game: &mut Game) {
        game.create_stage(FIRST);
        game.create_stage(UPDATE);
        game.create_stage(RENDER);
        game.create_stage(LAST);

        game.add_state(World::new());

        let threadpool = ThreadPoolBuilder::new()
            //.num_threads(2)
            .build()
            .expect("Failed to create threadpool");
        let threadpool = Arc::new(threadpool);

        game.add_state(threadpool.clone());
        let asset_manager = AssetManager::with_threadpool(threadpool);
        hikari_asset::serde::init(asset_manager.clone());

        game.add_state(AssetStorage::default());
        game.add_state(asset_manager);
    }
}
