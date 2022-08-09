use super::GizmoContext;

use super::{
    draw::Painter3D,
    math::{
        plane_binormal, plane_local_origin, plane_size, plane_tangent, ray_to_plane_origin,
        segment_to_segment, world_to_screen,
    },
    ray::Ray,
    subgizmo::SubGizmo,
};
use hikari_math::*;

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct ScaleState {
    start_scale: Vec3,
    start_delta: f32,
}

/// Picks given scale subgizmo. If the subgizmo is close enough to
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
    let ray_length = 1e+5;

    let (ray_t, subgizmo_t) = segment_to_segment(
        ray.origin,
        ray.origin + ray.direction * ray_length,
        origin,
        origin + dir * length,
    );

    let ray_point = ray.origin + ray.direction * ray_length * ray_t;
    let subgizmo_point = origin + dir * length * subgizmo_t;
    let dist = (ray_point - subgizmo_point).length();

    let start_delta = distance_from_origin_2d(subgizmo)?;

    *context.scale_state(subgizmo.id) = ScaleState {
        start_scale: subgizmo.state.transform.scale,
        start_delta: start_delta,
    };

    if dist <= subgizmo.state.focus_distance {
        Some(ray.origin.distance(ray_point))
    } else {
        None
    }
}
/// Picks given scale plane subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick_plane(
    subgizmo: &SubGizmo,
    context: &mut GizmoContext,
    ray: &Ray,
) -> Option<f32> {
    let origin = scale_plane_global_origin(subgizmo);

    let normal = subgizmo.normal();

    let (t, dist_from_origin) = ray_to_plane_origin(normal, origin, ray.origin, ray.direction);

    let start_delta = distance_from_origin_2d(subgizmo)?;

    *context.scale_state(subgizmo.id) = ScaleState {
        start_scale: subgizmo.state.transform.scale,
        start_delta: start_delta,
    };

    if dist_from_origin <= plane_size(subgizmo) {
        Some(t)
    } else {
        None
    }
}
/// Updates given scale subgizmo.
/// If the subgizmo is active, returns the scale result.
pub(crate) fn update_vector(
    subgizmo: &SubGizmo,
    context: &mut GizmoContext,
    _ray: &Ray,
) -> Option<Transform> {
    let state = context.scale_state(subgizmo.id);

    let current_state = *state;
    println!("{:#?}", state);

    let mut delta = distance_from_origin_2d(subgizmo)?;
    delta /= current_state.start_delta;

    // if subgizmo.config.snapping {
    //     delta = round_to_interval(delta, subgizmo.config.snap_scale);
    // }
    delta = delta.max(1e-4) - 1.0;

    let offset = Vec3::ONE + (subgizmo.local_normal() * delta);

    Some(Transform {
        scale: current_state.start_scale * offset,
        rotation: subgizmo.state.transform.rotation,
        position: subgizmo.state.transform.position,
    })
}

/// Updates given scale plane subgizmo.
/// If the subgizmo is active, returns the scale result.
pub(crate) fn update_plane(
    subgizmo: &SubGizmo,
    context: &mut GizmoContext,
    _ray: &Ray,
) -> Option<Transform> {
    let state = context.scale_state(subgizmo.id);
    let current_state = *state;

    let mut delta = distance_from_origin_2d(subgizmo)?;
    delta /= current_state.start_delta;

    // if subgizmo.config.snapping {
    //     delta = round_to_interval(delta, subgizmo.config.snap_scale);
    // }
    delta = delta.max(1e-4) - 1.0;

    let binormal = plane_binormal(subgizmo.direction);
    let tangent = plane_tangent(subgizmo.direction);
    let direction = (binormal + tangent).normalize();

    let offset = Vec3::ONE + (direction * delta);

    Some(Transform {
        scale: current_state.start_scale * offset,
        rotation: subgizmo.state.transform.rotation,
        position: subgizmo.state.transform.position,
    })
}

pub(crate) fn draw_vector(subgizmo: &SubGizmo, ui: &imgui::Ui) {
    let state = &subgizmo.state;
    let style = &subgizmo.style;
    let painter = Painter3D::new(
        ui,
        state.proj_view * scale_transform(subgizmo),
        state.viewport,
    );

    let vec = subgizmo.local_normal() * state.gizmo_size * state.scale_factor;

    let line_length = 1.0;
    let color = subgizmo.color();

    painter.line(Vec3::ZERO, vec * line_length, color, style.line_thickness);
}
pub(crate) fn draw_plane(subgizmo: &SubGizmo, ui: &imgui::Ui) {
    let state = &subgizmo.state;
    let painter = Painter3D::new(
        ui,
        state.proj_view * scale_transform(subgizmo),
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
fn scale_transform(subgizmo: &SubGizmo) -> Mat4 {
    if subgizmo.state.local_space() {
        Mat4::from_rotation_translation(
            subgizmo.state.transform.rotation,
            subgizmo.state.transform.position,
        )
    } else {
        Mat4::from_translation(subgizmo.state.transform.position)
    }
}
pub(crate) fn scale_plane_global_origin(subgizmo: &SubGizmo) -> Vec3 {
    let origin = plane_local_origin(subgizmo);
    subgizmo.state.transform.rotation * origin + subgizmo.state.transform.position
}
fn distance_from_origin_2d(subgizmo: &SubGizmo) -> Option<f32> {
    let viewport = subgizmo.state.viewport;
    let gizmo_pos = world_to_screen(subgizmo.state.mvp, viewport, Vec3::new(0.0, 0.0, 0.0))?;
    println!("Delta {}", subgizmo.state.cursor_pos.distance(gizmo_pos));
    Some(subgizmo.state.cursor_pos.distance(gizmo_pos))
}
