use __core::f32::consts::FRAC_PI_2;
use hikari::{
    input::KeyCode,
    math::{Quat, Transform, Vec2, Vec3},
};

use crate::imgui::*;

#[derive(Default)]
pub struct CameraState {
    rotation: Vec2,
}

pub fn manipulate(ui: &Ui, camera_state: &mut CameraState, transform: &mut Transform, dt: f32) {
    let speed = 4.5;
    let angular_speed = 0.25;
    let fast_multiplier = if ui.io().keys_down[KeyCode::LShift as usize] {
        3.0
    } else {
        1.0
    };

    if ui.io().keys_down[KeyCode::A as usize] {
        transform.position += -transform.right() * speed * fast_multiplier * dt;
    } else if ui.io().keys_down[KeyCode::D as usize] {
        transform.position += transform.right() * speed * fast_multiplier * dt;
    }

    if ui.io().keys_down[KeyCode::W as usize] {
        transform.position += transform.forward() * speed * fast_multiplier * dt;
    } else if ui.io().keys_down[KeyCode::S as usize] {
        transform.position += -transform.forward() * speed * fast_multiplier * dt;
    }

    if ui.io().keys_down[KeyCode::E as usize] {
        transform.position += transform.up() * speed * fast_multiplier * dt;
    } else if ui.io().keys_down[KeyCode::Q as usize] {
        transform.position += -transform.up() * speed * fast_multiplier * dt;
    }

    if ui.io().mouse_down[MouseButton::Middle as usize] {
        let rotation = &mut camera_state.rotation;

        let delta = ui.io().mouse_delta;

        rotation.x += delta[0] * angular_speed * dt;
        rotation.y += delta[1] * angular_speed * dt;

        rotation.y = rotation.y.clamp(-FRAC_PI_2, FRAC_PI_2);

        transform.rotation = Quat::from_axis_angle(-Vec3::Y, rotation.x);
        transform.rotation *= Quat::from_axis_angle(-Vec3::X, rotation.y);
    }
}
