use hikari::core::serde::{Registry, SerializeComponent};

use crate::components::{EditorComponent, EditorComponents};

mod camera;
mod light;
mod mesh_render;
mod meta;
mod transform;
mod environment;

fn register_component<C: EditorComponent + SerializeComponent>(
    components: &mut EditorComponents,
    registry: &mut Registry,
) {
    components.register::<C>();
    registry.register_component::<C>();
}
pub fn register_components(components: &mut EditorComponents, registry: &mut Registry) {
    register_component::<hikari::math::Transform>(components, registry);
    register_component::<hikari::g3d::Camera>(components, registry);
    register_component::<hikari::g3d::MeshRender>(components, registry);
    register_component::<hikari::g3d::Light>(components, registry);
    register_component::<hikari::g3d::Environment>(components, registry);

    register_component::<crate::editor::meta::EditorInfo>(components, registry);
    register_component::<crate::editor::meta::EditorOnly>(components, registry);
}
