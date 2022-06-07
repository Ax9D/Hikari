use crate::{GizmoContext, gizmo::math::plane_global_origin};

use super::{
    draw::Painter3D,
    math::{intersect_plane, ray_to_plane_origin, ray_to_ray, segment_to_segment, plane_local_origin, plane_tangent, plane_binormal, plane_size},
    ray::Ray,
    subgizmo::SubGizmo,
};
use hikari_math::*;

#[derive(Default, Copy, Clone, Debug)]
pub(crate) struct TranslationState {
    start_point: Vec3,
    last_point: Vec3,
    current_delta: Vec3,
}

/// Picks given translation subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick_vector(
    subgizmo: &SubGizmo,
    context: &mut GizmoContext,
    ray: &Ray,
) -> Option<f32> {
    let origin = subgizmo.state.transform.position;
    let dir = subgizmo.normal();
    let scale = subgizmo.state.scale_factor * subgizmo.state.gizmo_size;
    let length = scale;
    let ray_length = 10000.0;

    let (ray_t, subgizmo_t) = segment_to_segment(
        ray.origin,
        ray.origin + ray.direction * ray_length,
        origin,
        origin + dir * length,
    );

    let ray_point = ray.origin + ray.direction * ray_length * ray_t;
    let subgizmo_point = origin + dir * length * subgizmo_t;
    let dist = (ray_point - subgizmo_point).length();

    *context.translation_state(subgizmo.id) = TranslationState {
        start_point: subgizmo_point,
        last_point: subgizmo_point,
        current_delta: Vec3::ZERO,
    };

    if dist <= subgizmo.state.focus_distance {

        Some(ray.origin.distance(ray_point))
    } else {
        None
    }
}
/// Updates given translation subgizmo.
/// If the subgizmo is active, returns the translation result.
pub(crate) fn update_vector(
    subgizmo: &SubGizmo,
    context: &mut GizmoContext,
    ray: &Ray,
) -> Option<Transform> {
    let state = context.translation_state(subgizmo.id);
    let current_state = *state;

    let new_point = point_on_axis(subgizmo, *ray);
    let new_delta = new_point - current_state.start_point;

    // if subgizmo.state.snapping {
    //     new_delta = snap_translation_vector(subgizmo, new_delta);
    //     new_point = state.start_point + new_delta;
    // }

    state.last_point = new_point;
    state.current_delta = new_delta;
    Some(Transform {
        position: subgizmo.state.transform.position + new_point - current_state.last_point,
        scale: subgizmo.state.transform.scale,
        rotation: subgizmo.state.transform.rotation,
    })
}
fn translation_transform(subgizmo: &SubGizmo) -> Mat4 {
    if subgizmo.state.local_space() {
        Mat4::from_rotation_translation(
            subgizmo.state.transform.rotation,
            subgizmo.state.transform.position,
        )
    } else {
        Mat4::from_translation(subgizmo.state.transform.position)
    }
}
/// Finds the nearest point on line that points in translation subgizmo direction
fn point_on_axis(subgizmo: &SubGizmo, ray: Ray) -> Vec3 {
    let origin = subgizmo.state.transform.position;
    let direction = subgizmo.normal();

    let (_ray_t, subgizmo_t) = ray_to_ray(ray.origin, ray.direction, origin, direction);

    origin + direction * subgizmo_t
}
/// Picks given translation plane subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick_plane(
    subgizmo: &SubGizmo,
    context: &mut GizmoContext,
    ray: &Ray,
) -> Option<f32> {
    let origin = plane_global_origin(subgizmo);

    let normal = subgizmo.normal();

    let (t, dist_from_origin) = ray_to_plane_origin(normal, origin, ray.origin, ray.direction);

    let ray_point = ray.origin + ray.direction * t;

    *context.translation_state(subgizmo.id) = TranslationState {
        start_point: ray_point,
        last_point: ray_point,
        current_delta: Vec3::ZERO,
    };

    if dist_from_origin <= plane_size(subgizmo) {
        Some(t)
    } else {
        None
    }
}
/// Updates given translation subgizmo.
/// If the subgizmo is active, returns the translation result.
pub(crate) fn update_plane(
    subgizmo: &SubGizmo,
    context: &mut GizmoContext,
    ray: &Ray,
) -> Option<Transform> {
    let state = context.translation_state(subgizmo.id);
    let current_state = *state;

    let new_point = point_on_plane(
        subgizmo.normal(),
        plane_global_origin(subgizmo),
        *ray,
    )?;

    let new_delta = new_point - current_state.start_point;

    // if subgizmo.config.snapping {
    //     new_delta = snap_translation_plane(subgizmo, new_delta);
    //     new_point = state.start_point + new_delta;
    // }

    state.last_point = new_point;
    state.current_delta = new_delta;

    Some(Transform {
        position: subgizmo.state.transform.position + new_point - current_state.last_point,
        rotation: subgizmo.state.transform.rotation,
        scale: subgizmo.state.transform.scale,
    })
}
fn point_on_plane(plane_normal: Vec3, plane_origin: Vec3, ray: Ray) -> Option<Vec3> {
    let mut t = 0.0;
    if !intersect_plane(
        plane_normal,
        plane_origin,
        ray.origin,
        ray.direction,
        &mut t,
    ) {
        None
    } else {
        Some(ray.origin + ray.direction * t)
    }
}
pub(crate) fn draw_vector(subgizmo: &SubGizmo, ui: &imgui::Ui) {
    let state = &subgizmo.state;
    let style = &subgizmo.style;
    let painter = Painter3D::new(
        ui,
        state.proj_view * translation_transform(subgizmo),
        state.viewport,
    );

    let vec = subgizmo.local_normal() * state.gizmo_size * state.scale_factor;

    const ARROW_LENGTH: f32 = 0.2;
    let line_length = 1.0 - ARROW_LENGTH;
    let color = subgizmo.color();

    painter.line(Vec3::ZERO, vec * line_length, color, style.line_thickness);
    painter.arrowhead(vec * line_length, vec, color, style.line_thickness);
}
pub(crate) fn draw_plane(subgizmo: &SubGizmo, ui: &imgui::Ui) {
    let state = &subgizmo.state;
    let painter = Painter3D::new(
        ui,
        state.proj_view * translation_transform(subgizmo),
        state.viewport,
    );

    let color = subgizmo.color();

    let scale = plane_size(subgizmo) * 0.5;
    let a = plane_binormal(subgizmo.direction) * scale;
    let b = plane_tangent(subgizmo.direction) * scale;

    let origin = plane_local_origin(subgizmo);

    painter.polygon(
        &[
            origin - b - a,
            origin + b - a,
            origin + b + a,
            origin - b + a,
        ],
        color,
    );
}
