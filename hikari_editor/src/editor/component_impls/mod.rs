mod transform;
mod mesh_render;
mod camera;
mod light;

use super::components::EditorComponents;

pub fn register_components(components: &mut EditorComponents) {
    components.register::<hikari::math::Transform>();
    components.register::<hikari::g3d::Camera>();
    components.register::<hikari::g3d::MeshRender>();
    components.register::<hikari::g3d::Light>();
}
