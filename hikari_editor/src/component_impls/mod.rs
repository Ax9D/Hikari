use hikari::core::{serialize::SerializeComponent, CloneComponent, RegistryBuilder};

use crate::{components::{EditorComponent, EditorComponents}, editor};

mod camera;
mod environment;
mod light;
mod mesh_render;
mod meta;
mod transform;

fn register_editor_serde_clone<C: EditorComponent + SerializeComponent + CloneComponent>(
    components: &mut EditorComponents,
    registry: &mut RegistryBuilder,
) {
    components.register::<C>();
    registry.register_clone::<C>();
    registry.register_serde::<C>();
}
fn register_serde_and_clone<C: SerializeComponent + CloneComponent>(registry: &mut RegistryBuilder) {
    registry.register_serde::<C>();
    registry.register_clone::<C>();
}
pub fn register_components(components: &mut EditorComponents, registry: &mut RegistryBuilder) {
    register_editor_serde_clone::<hikari::math::Transform>(components, registry);
    register_editor_serde_clone::<hikari::g3d::Camera>(components, registry);
    register_editor_serde_clone::<hikari::g3d::MeshRender>(components, registry);
    register_editor_serde_clone::<hikari::g3d::Light>(components, registry);
    register_editor_serde_clone::<hikari::g3d::Environment>(components, registry);

    register_serde_and_clone::<editor::meta::EditorOnly>(registry);
    register_serde_and_clone::<editor::meta::EditorOutlinerInfo>(registry);
    register_serde_and_clone::<editor::camera::ViewportCamera>(registry);
}
