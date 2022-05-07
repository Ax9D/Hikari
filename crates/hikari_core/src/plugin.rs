use crate::Game;

#[allow(unused_variables)]
pub trait Plugin {
    fn build(self, game: &mut Game);
}
