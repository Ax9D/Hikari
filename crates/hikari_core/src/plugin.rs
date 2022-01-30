use crate::Game;

#[allow(unused_variables)]
pub trait Plugin {
    fn build(&mut self, game: &mut Game);
}
