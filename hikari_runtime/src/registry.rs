use hikari::core::{Registry, RegistryBuilder};
use hikari::core::serialize::SerializeComponent;
use hikari::core::CloneComponent;

fn register_serde_and_clone<C: SerializeComponent + CloneComponent>(registry: &mut RegistryBuilder) {
    registry.register_serde::<C>();
    registry.register_clone::<C>();
}
pub fn default_registry() -> Registry {
    let mut registry = Registry::builder();

    register_serde_and_clone::<hikari::math::Transform>(&mut registry);
    register_serde_and_clone::<hikari::g3d::Camera>(&mut registry);
    register_serde_and_clone::<hikari::g3d::MeshRender>(&mut registry);
    register_serde_and_clone::<hikari::g3d::Light>(&mut registry);
    register_serde_and_clone::<hikari::g3d::Environment>(&mut registry);

    registry.build()
}