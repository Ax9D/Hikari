use crate::core::primitives::Transform;

pub struct CameraComponent {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub primary: bool,
}
impl CameraComponent {
    pub fn new(fov: f32, near: f32, far: f32, primary: bool) -> Self {
        Self {
            fov,
            near,
            far,
            primary,
        }
    }
    pub fn get_view_matrix(&self, transform: &Transform) -> glam::Mat4 {
        // let yaw = transform.rotation.y;
        // let pitch = transform.rotation.x;
        // let roll = transform.rotation.z;

        let view = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::ONE,
            transform.rotation,
            transform.position,
        );
        view.inverse()
    }
    pub fn get_projection_matrix(&self, aspect_ratio: f32) -> glam::Mat4 {
        glam::Mat4::perspective_rh_gl(self.fov, aspect_ratio, self.near, self.far)
    }
}
