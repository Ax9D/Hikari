mod ecs;
mod game;
mod plugin;
mod window;

pub use ecs::*;
pub use game::*;
pub use plugin::*;

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
    }
}
