use super::GizmoContext;

use super::{
    draw::Painter3D,
    math::{ray_to_plane_origin, rotation_align, world_to_screen},
    ray::Ray,
    subgizmo::SubGizmo,
    Direction,
};
use hikari_math::*;
use imgui::ImColor32;

use std::f32::consts::*;

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct RotationState {
    start_axis_angle: f32,
    #[allow(dead_code)]
    start_rotation_angle: f32,
    last_rotation_angle: f32,
    current_delta: f32,
}

/// Picks given rotation subgizmo. If the subgizmo is close enough to
/// the mouse pointer, distance from camera to the subgizmo is returned.
pub(crate) fn pick(subgizmo: &SubGizmo, context: &mut GizmoContext, ray: &Ray) -> Option<f32> {
    let radius = arc_radius(subgizmo);
    let state = subgizmo.state;
    let origin = state.transform.position;
    let normal = subgizmo.normal();
    let tangent = tangent(subgizmo);

    let (t, dist_from_gizmo_origin) =
        ray_to_plane_origin(normal, origin, ray.origin, ray.direction);
    let dist_from_gizmo_edge = (dist_from_gizmo_origin - radius).abs();

    let hit_pos = ray.origin + ray.direction * t;
    let dir_to_origin = (origin - hit_pos).normalize();
    let nearest_circle_pos = hit_pos + dir_to_origin * (dist_from_gizmo_origin - radius);

    let offset = (nearest_circle_pos - origin).normalize();

    let angle = if subgizmo.direction == Direction::Screen {
        f32::atan2(tangent.cross(normal).dot(offset), tangent.dot(offset))
    } else {
        let forward = state.view_forward();
        f32::atan2(offset.cross(forward).dot(normal), offset.dot(forward))
    };

    let rotation_angle = rotation_angle(subgizmo).unwrap_or(0.0);
    *context.rotation_state(subgizmo.id) = RotationState {
        start_axis_angle: angle,
        start_rotation_angle: rotation_angle,
        last_rotation_angle: rotation_angle,
        current_delta: 0.0,
    };

    println!("arc_angle {}", arc_angle(subgizmo).to_degrees());
    if dist_from_gizmo_edge <= state.focus_distance && angle.abs() < arc_angle(subgizmo) {
        Some(t)
    } else {
        None
    }
}

/// Updates given rotation subgizmo.
/// If the subgizmo is active, returns the rotation result.
pub(crate) fn update(
    subgizmo: &SubGizmo,
    context: &mut GizmoContext,
    _ray: &Ray,
) -> Option<Transform> {
    let state = context.rotation_state(subgizmo.id);
    let current_state = *state;

    let rotation_angle = rotation_angle(subgizmo)?;
    // if config.snapping {
    //     rotation_angle = round_to_interval(
    //         rotation_angle - state.start_rotation_angle,
    //         config.snap_angle,
    //     ) + state.start_rotation_angle;
    // }

    let mut angle_delta = rotation_angle - current_state.last_rotation_angle;

    // Always take the smallest angle, e.g. -10° instead of 350°
    if angle_delta > PI {
        angle_delta -= TAU;
    } else if angle_delta < -PI {
        angle_delta += TAU;
    }

    state.last_rotation_angle = rotation_angle;
    state.current_delta += angle_delta;

    let rotation =
        Quat::from_axis_angle(subgizmo.normal(), -angle_delta) * subgizmo.state.transform.rotation;

    Some(Transform {
        rotation,
        scale: subgizmo.state.transform.scale,
        position: subgizmo.state.transform.position,
    })
}
pub(crate) fn draw(subgizmo: &SubGizmo, context: &mut GizmoContext, ui: &imgui::Ui) {
    //let state = subgizmo.state::<RotationState>(ui);
    let transform = rotation_matrix(subgizmo);
    let painter = Painter3D::new(
        ui,
        subgizmo.state.proj_view * transform,
        subgizmo.state.viewport,
    );

    let color = subgizmo.color();
    let fill_color = ImColor32::from_rgba(color.r, color.g, color.b, 10);

    let radius = arc_radius(subgizmo);

    if !subgizmo.active {
        let angle = arc_angle(subgizmo);
        painter.arc(
            radius,
            FRAC_PI_2 - angle,
            FRAC_PI_2 + angle,
            color,
            subgizmo.style.line_thickness,
        );
        //painter.circle(radius, color, subgizmo.style.line_thickness);
    } else {
        let state = context.rotation_state(subgizmo.id);
        let start_angle = state.start_axis_angle + FRAC_PI_2;
        let end_angle = start_angle + state.current_delta;

        // The polyline does not get rendered correctly if
        // the start and end lines are exactly the same
        let end_angle = end_angle + 1e-5;

        painter.polyline(
            &[
                Vec3::new(start_angle.cos() * radius, 0.0, start_angle.sin() * radius),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(end_angle.cos() * radius, 0.0, end_angle.sin() * radius),
            ],
            color,
            subgizmo.style.line_thickness,
        );

        painter.sector(
            radius,
            start_angle,
            end_angle,
            fill_color,
            subgizmo.style.line_thickness,
        );

        painter.circle(radius, color, subgizmo.style.line_thickness);

        drop(painter);
        let delta_angles = subgizmo.local_normal() * state.current_delta;
        let delta_string = format!(
            "dx: {}° dy: {}° dz: {}°",
            delta_angles.x.to_degrees(),
            delta_angles.y.to_degrees(),
            delta_angles.z.to_degrees()
        );
        ui.get_window_draw_list().add_text(
            subgizmo.state.viewport.max - Vec2::new(150.0, 75.0),
            ImColor32::WHITE,
            &delta_string,
        );
        // // Draw snapping ticks
        // if config.snapping {
        //     let stroke_width = stroke.0 / 2.0;
        //     for i in 0..((TAU / config.snap_angle) as usize + 1) {
        //         let angle = i as f32 * config.snap_angle + end_angle;
        //         let pos = Vec3::new(angle.cos(), 0.0, angle.sin());
        //         painter.line_segment(
        //             pos * radius * 1.1,
        //             pos * radius * 1.2,
        //             (stroke_width, stroke.1),
        //         );
        //     }
        // }
    }
}

/// Calculates angle of the rotation axis arc.
/// The arc is a semicircle, which turns into a full circle when viewed
/// directly from the front.
fn arc_angle(subgizmo: &SubGizmo) -> f32 {
    let dot = subgizmo.normal().dot(subgizmo.state.view_forward()).abs();
    let min_dot = 0.990;
    let max_dot = 0.995;

    f32::min(1.0, f32::max(0.0, dot - min_dot) / (max_dot - min_dot)) * FRAC_PI_2 + FRAC_PI_2
}

/// Calculates a matrix used when rendering the rotation axis.
fn rotation_matrix(subgizmo: &SubGizmo) -> Mat4 {
    // First rotate towards the gizmo normal
    let local_normal = subgizmo.local_normal();
    let rotation = rotation_align(Vec3::Y, local_normal);
    let mut rotation = Quat::from_mat3(&rotation);
    let state = subgizmo.state;

    // TODO optimize this. Use same code for all axes if possible.

    if subgizmo.direction != Direction::Screen {
        if state.local_space() {
            rotation = state.transform.rotation * rotation;
        }

        let tangent = tangent(subgizmo);
        let normal = subgizmo.normal();
        let forward = state.view_forward();
        let angle = f32::atan2(tangent.cross(forward).dot(normal), tangent.dot(forward));

        // Rotate towards the camera, along the rotation axis.
        rotation = Quat::from_axis_angle(normal, angle) * rotation;
    } else {
        let angle = f32::atan2(local_normal.x, local_normal.z) + FRAC_PI_2;
        rotation = Quat::from_axis_angle(local_normal, angle) * rotation;
    }

    Mat4::from_rotation_translation(rotation, state.transform.position)
}

fn rotation_angle(subgizmo: &SubGizmo) -> Option<f32> {
    let cursor_pos = subgizmo.state.cursor_pos;
    let viewport = subgizmo.state.viewport;
    let gizmo_pos = world_to_screen(subgizmo.state.mvp, viewport, Vec3::new(0.0, 0.0, 0.0))?;
    let delta = Vec2::new(cursor_pos.x - gizmo_pos.x, cursor_pos.y - gizmo_pos.y).normalize();

    if delta.is_nan() {
        return None;
    }

    let mut angle = f32::atan2(delta.y, delta.x);
    if subgizmo.state.view_forward().dot(subgizmo.normal()) < 0.0 {
        angle *= -1.0;
    }

    Some(angle)
}
fn tangent(subgizmo: &SubGizmo) -> Vec3 {
    let mut tangent = match subgizmo.direction {
        Direction::X => Vec3::Z,
        Direction::Y => Vec3::Z,
        Direction::Z => -Vec3::Y,
        Direction::Screen => -subgizmo.state.view_right(),
    };

    if subgizmo.state.local_space() && subgizmo.direction != Direction::Screen {
        tangent = subgizmo.state.transform.rotation * tangent;
    }

    tangent
}
fn arc_radius(subgizmo: &SubGizmo) -> f32 {
    let mut radius = subgizmo.state.gizmo_size;

    if subgizmo.direction == Direction::Screen {
        // Screen axis should be a little bit larger
        radius += subgizmo.style.line_thickness + 5.0;
    }

    subgizmo.state.scale_factor * radius
}
