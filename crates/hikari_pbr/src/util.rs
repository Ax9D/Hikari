use hikari_3d::*;
use hikari_core::*;
use hikari_math::*;

pub fn get_directional_light(world: &World) -> Option<Entity> {
    for (entity, pair) in &mut world.query::<(&Light, &Transform)>() {
        if matches!(pair.0.kind, LightKind::Directional) {
            return Some(entity);
        }
    }

    None
}
pub fn get_camera(world: &World) -> Option<Entity> {
    world
        .query::<(&Camera, &Transform)>()
        .iter()
        .filter(|(_, (camera, _))| camera.is_primary)
        .next()
        .map(|(entity, _)| entity)
}
