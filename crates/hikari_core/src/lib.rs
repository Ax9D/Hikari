mod ecs;
mod game;
mod plugin;
mod time;
mod window;

use std::sync::Arc;

pub use ecs::*;
pub use game::*;
pub use time::*;

pub use hikari_systems::*;
pub use plugin::*;
use rayon::ThreadPoolBuilder;

pub use winit;

pub const FIRST: &'static str = "First";
pub const UPDATE: &'static str = "Update";
pub const RENDER: &'static str = "Render";
pub const POST_RENDER: &'static str = "PostRender";
pub const LAST: &'static str = "Last";
pub struct CorePlugin;

impl crate::Plugin for CorePlugin {
    fn build(self, game: &mut Game) {
        game.create_stage(FIRST);
        game.create_stage(UPDATE);
        game.create_stage(RENDER);
        game.create_stage(POST_RENDER);
        game.create_stage(LAST);

        game.add_state(Time::new());

        game.add_task(
            FIRST,
            Task::new("Update Delta Time", |time: &mut Time| {
                time.update();
            }),
        );

        game.add_state(World::new());

        let threadpool = ThreadPoolBuilder::new()
            //.num_threads(2)
            .build()
            .expect("Failed to create threadpool");
        let threadpool = Arc::new(threadpool);

        game.add_state(threadpool.clone());
    }
}
