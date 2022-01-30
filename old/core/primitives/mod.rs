pub mod identifier;
//pub mod shape;
// pub mod sprite;
pub mod scene;
pub mod transform;

pub use scene::Scene;
pub use transform::Transform;

pub use scene::Entity;
pub trait CoreSystem {
    type State;

    fn preInit(app: &mut crate::core::App) -> Result<Self::State, Box<dyn std::error::Error>>;
    fn postInit(ctx: &mut crate::core::Context) {}
}
