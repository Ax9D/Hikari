use hikari::math::{Quat, Transform, Vec2, Vec3};

use hikari::imgui::*;

pub struct ViewportCamera {
    rotation: Vec2,
    pub speed: f32,
    pub angular_speed: f32,
}

impl Default for ViewportCamera {
    fn default() -> Self {
        Self {
            rotation: Vec2::ZERO,
            speed: 4.5,
            angular_speed: 0.25,
        }
    }
}

impl ViewportCamera {
    pub fn manipulate(&mut self, ui: &Ui, transform: &mut Transform, dt: f32) {
        hikari::dev::profile_function!();

        let speed = self.speed;
        let sensitivity = self.angular_speed;

        //let externally_changed = !as_quat.abs_diff_eq(transform.rotation, 0.001)

        //dbg!(x.to_degrees(), y.to_degrees(), z.to_degrees());

        // if externally_changed {
        //     let (x, y, z) =  transform.rotation.to_euler(EulerRot::XYZ);
        //     self.rotation.x = -y;
        //     self.rotation.y = z;
        // }

        let fast_multiplier = if ui.is_key_down(Key::LeftShift) {
            3.0
        } else {
            1.0
        };

        if ui.is_key_down(Key::A) {
            transform.position += -transform.right() * speed * fast_multiplier * dt;
        } else if ui.is_key_down(Key::D) {
            transform.position += transform.right() * speed * fast_multiplier * dt;
        }
        if ui.is_key_down(Key::W) {
            transform.position += transform.forward() * speed * fast_multiplier * dt;
        } else if ui.is_key_down(Key::S) {
            transform.position += -transform.forward() * speed * fast_multiplier * dt;
        }

        if ui.is_key_down(Key::E) {
            transform.position += transform.up() * speed * fast_multiplier * dt;
        } else if ui.is_key_down(Key::Q) {
            transform.position += -transform.up() * speed * fast_multiplier * dt;
        }

        if ui.is_mouse_down(MouseButton::Middle) {
            let rotation = &mut self.rotation;
            let mouse_delta: Vec2 = ui.io().mouse_delta.into();
            let delta: Vec2 = mouse_delta * sensitivity * dt;

            *rotation += delta;

            let delta_quat = Quat::from_axis_angle(Vec3::Y, rotation.x)
                * Quat::from_axis_angle(Vec3::X, rotation.y);

            //*rotation *= delta_quat;

            //let (mut rot_x, rot_y, rot_z) =rotation.to_euler(EulerRot::XYZ);
            //if rot_x > 180.0 {
            //    rot_x -= 360.0;
            //}
            //let mut rot_x = rot_x.clamp(-FRAC_PI_2 + 10.0f32.to_radians(), FRAC_PI_2 - 10.0f32.to_radians());
            //*rotation = Quat::from_euler(EulerRot::XYZ, rot_x, rot_y, 0.0).normalize();

            transform.rotation = delta_quat;
        }
    }
}
