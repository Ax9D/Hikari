mod ecs;
mod game;
mod math;
mod plugin;
mod window;

pub use ecs::*;
pub use game::*;
pub use math::*;
pub use plugin::*;

pub struct Core;
use hikari_systems::Stage;

impl Plugin for Core {
    fn build(&mut self, game: &mut Game) {
        let first = Stage::new("First");
        let mut update = Stage::new("Update");
        let mut render = Stage::new("Render");
        let mut last = Stage::new("Last");

        update.after(first.name());
        render.after(update.name());
        last.after(render.name());

        game.add_stage(first);
        game.add_stage(update);
        game.add_stage(render);
        game.add_stage(last);
    }
}
