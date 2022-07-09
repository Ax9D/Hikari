#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub near: f32,
    pub far: f32,
    pub exposure: f32,
    pub projection: Projection,
}

impl Default for Camera {
    fn default() -> Self {
        Self { near: 0.1,
        far: 1000.0,
    exposure: 1.0,
projection: Projection::Perspective(45.0) }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Projection {
    Perspective(f32), //fov in degrees
    Orthographic,
}


impl Camera {
    pub fn get_projection_matrix(&self, width: u32, height: u32) -> hikari_math::Mat4 {
        match self.projection {
            Projection::Perspective(fov) => hikari_math::Mat4::perspective_rh(
                fov.to_radians(),
                width as f32 / height as f32,
                self.near,
                self.far,
            ),
            Projection::Orthographic => todo!(),
        }
    }
}
