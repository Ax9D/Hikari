use hikari_math::*;

use crate::Viewport;

use super::{subgizmo::SubGizmo, Direction};

pub(crate) fn world_to_screen(mvp: Mat4, viewport: Viewport, pos: Vec3) -> Option<Vec2> {
    let mut pos = mvp * Vec4::from((pos, 1.0));

    if pos.w < 0.0 {
        return None;
    }

    pos /= pos.w;
    pos.y *= -1.0;

    let center = viewport.center();
    Some(Vec2::new(
        center.x + pos.x * viewport.width() / 2.0,
        center.y + pos.y * viewport.height() / 2.0,
    ))
}
/// Creates a matrix that represents rotation between two 3d vectors
///
/// Credit: https://www.iquilezles.org/www/articles/noacos/noacos.htm
pub fn rotation_align(from: Vec3, to: Vec3) -> Mat3 {
    let v = from.cross(to);
    let c = from.dot(to);
    let k = 1.0 / (1.0 + c);

    Mat3::from_cols_array(&[
        v.x * v.x * k + c,
        v.x * v.y * k + v.z,
        v.x * v.z * k - v.y,
        v.y * v.x * k - v.z,
        v.y * v.y * k + c,
        v.y * v.z * k + v.x,
        v.z * v.x * k + v.y,
        v.z * v.y * k - v.x,
        v.z * v.z * k + c,
    ])
}

/// Finds points on two rays that are closest to each other.
/// This can be used to determine the shortest distance between those two rays.
///
/// Credit: Practical Geometry Algorithms by Daniel Sunday: http://geomalgorithms.com/code.html
pub fn ray_to_ray(a1: Vec3, adir: Vec3, b1: Vec3, bdir: Vec3) -> (f32, f32) {
    let b = adir.dot(bdir);
    let w = a1 - b1;
    let d = adir.dot(w);
    let e = bdir.dot(w);
    let dot = 1.0 - b * b;
    let ta;
    let tb;

    if dot < 1e-8 {
        ta = 0.0;
        tb = e;
    } else {
        ta = (b * e - d) / dot;
        tb = (e - b * d) / dot;
    }

    (ta, tb)
}
/// Finds points on two segments that are closest to each other.
/// This can be used to determine the shortest distance between those two segments.
///
/// Credit: Practical Geometry Algorithms by Daniel Sunday: http://geomalgorithms.com/code.html
pub fn segment_to_segment(a1: Vec3, a2: Vec3, b1: Vec3, b2: Vec3) -> (f32, f32) {
    let da = a2 - a1;
    let db = b2 - b1;
    let la = da.length_squared();
    let lb = db.length_squared();
    let dd = da.dot(db);
    let d1 = a1 - b1;
    let d = da.dot(d1);
    let e = db.dot(d1);
    let n = la * lb - dd * dd;

    let mut sn;
    let mut tn;
    let mut sd = n;
    let mut td = n;

    if n < 1e-8 {
        sn = 0.0;
        sd = 1.0;
        tn = e;
        td = lb;
    } else {
        sn = dd * e - lb * d;
        tn = la * e - dd * d;
        if sn < 0.0 {
            sn = 0.0;
            tn = e;
            td = lb;
        } else if sn > sd {
            sn = sd;
            tn = e + dd;
            td = lb;
        }
    }

    if tn < 0.0 {
        tn = 0.0;
        if -d < 0.0 {
            sn = 0.0;
        } else if -d > la {
            sn = sd;
        } else {
            sn = -d;
            sd = la;
        }
    } else if tn > td {
        tn = td;
        if (-d + dd) < 0.0 {
            sn = 0.0;
        } else if (-d + dd) > la {
            sn = sd;
        } else {
            sn = -d + dd;
            sd = la;
        }
    }

    let ta = if sn.abs() < 1e-8 { 0.0 } else { sn / sd };
    let tb = if tn.abs() < 1e-8 { 0.0 } else { tn / td };

    (ta, tb)
}

/// Finds the intersection point of a ray and a plane
pub fn intersect_plane(
    plane_normal: Vec3,
    plane_origin: Vec3,
    ray_origin: Vec3,
    ray_dir: Vec3,
    t: &mut f32,
) -> bool {
    let denom = plane_normal.dot(ray_dir);

    if denom.abs() < 10e-8 {
        false
    } else {
        *t = (plane_origin - ray_origin).dot(plane_normal) / denom;
        *t >= 0.0
    }
}

/// Finds the intersection point of a ray and a plane
/// and distance from the intersection to the plane origin
pub fn ray_to_plane_origin(
    disc_normal: Vec3,
    disc_origin: Vec3,
    ray_origin: Vec3,
    ray_dir: Vec3,
) -> (f32, f32) {
    let mut t = 0.0;
    if intersect_plane(disc_normal, disc_origin, ray_origin, ray_dir, &mut t) {
        let p = ray_origin + ray_dir * t;
        let v = p - disc_origin;
        let d2 = v.dot(v);
        (t, f32::sqrt(d2))
    } else {
        (t, f32::MAX)
    }
}

pub(crate) fn plane_binormal(direction: Direction) -> Vec3 {
    match direction {
        Direction::X => Vec3::Y,
        Direction::Y => Vec3::Z,
        Direction::Z => Vec3::X,
        Direction::Screen => unreachable!(),
    }
}

pub(crate) fn plane_tangent(direction: Direction) -> Vec3 {
    match direction {
        Direction::X => Vec3::Z,
        Direction::Y => Vec3::X,
        Direction::Z => Vec3::Y,
        Direction::Screen => unreachable!(),
    }
}
pub(crate) fn plane_size(subgizmo: &SubGizmo) -> f32 {
    subgizmo.state.scale_factor
        * (subgizmo.state.gizmo_size * 0.1 + subgizmo.style.line_thickness * 2.0)
}

pub(crate) fn plane_local_origin(subgizmo: &SubGizmo) -> Vec3 {
    let offset = subgizmo.state.scale_factor * subgizmo.state.gizmo_size * 0.4;

    let a = plane_binormal(subgizmo.direction);
    let b = plane_tangent(subgizmo.direction);
    (a + b) * offset
}

pub(crate) fn plane_global_origin(subgizmo: &SubGizmo) -> Vec3 {
    let mut origin = plane_local_origin(subgizmo);
    if subgizmo.state.local_space() {
        origin = subgizmo.state.transform.rotation * origin;
    }
    origin + subgizmo.state.transform.position
}
