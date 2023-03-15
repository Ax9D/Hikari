use hikari::core::{serialize::SerializeComponent, CloneComponent, RegistryBuilder};

use crate::components::{EditorComponent, EditorComponents};

mod camera;
mod environment;
mod light;
mod mesh_render;
mod meta;
mod transform;

fn register_component<C: EditorComponent + SerializeComponent + CloneComponent>(
    components: &mut EditorComponents,
    registry: &mut RegistryBuilder,
) {
    components.register::<C>();
    registry.register_clone::<C>();
    registry.register_serde::<C>();
}
pub fn register_components(components: &mut EditorComponents, registry: &mut RegistryBuilder) {
    register_component::<hikari::math::Transform>(components, registry);
    register_component::<hikari::g3d::Camera>(components, registry);
    register_component::<hikari::g3d::MeshRender>(components, registry);
    register_component::<hikari::g3d::Light>(components, registry);
    register_component::<hikari::g3d::Environment>(components, registry);

    register_component::<crate::editor::meta::EditorOnly>(components, registry);
    register_component::<crate::editor::meta::EditorInfo>(components, registry);
}
