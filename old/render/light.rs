#[repr(C)]
pub struct DirectionalLight {
    pub intensity: f32,
    pub color: glam::Vec3,
}
impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            intensity: 2.0,
            color: glam::vec3(1.0, 1.0, 1.0),
        }
    }
}

impl DirectionalLight {
    pub fn calculate_direction(rotation: &glam::Vec3) -> glam::Vec3 {
        let yaw = rotation.y;
        let pitch = rotation.x;
        let roll = rotation.z;

        let x = yaw.cos() * pitch.cos();
        let y = yaw.sin() * pitch.cos();
        let z = pitch.sin();

        glam::vec3(x, y, z)
    }
}
