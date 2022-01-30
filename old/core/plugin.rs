use crate::window::Event;

pub trait Plugin {
    fn on_init(ctx: &mut crate::Context) -> Self
    where
        Self: Sized;
    fn on_update(&mut self, ctx: &mut crate::Context, dt: f32);
    fn on_event(&mut self, ctx: &mut crate::Context, event: &mut Event);
}
