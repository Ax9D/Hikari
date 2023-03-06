use hikari_math::{Quat, Transform, Vec3};

//Creates a correction matrix for right handed transformation to left handed transformation assuming the following convention:
// Right Handed: +x = right, +y = up, +z = forward
// Left Handed:  +x = right, +y = up, -z = forward
pub fn left_handed_correction(transform: Transform) -> Transform {
    let position = transform.position;

    let scale = Vec3::new(transform.scale.x, transform.scale.y, -transform.scale.z);

    let (axis, angle) = transform.rotation.to_axis_angle();
    let rotation = Quat::from_axis_angle(Vec3::new(axis.x, axis.y, -axis.z), -angle);

    Transform {
        position,
        scale,
        rotation,
    }
}
pub fn ccw_to_cw<T>(arr: &mut [T]) {
    arr.chunks_exact_mut(3).for_each(|tri| tri.reverse())
}
