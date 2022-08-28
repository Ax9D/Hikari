#[derive(Clone, Copy, Debug, type_uuid::TypeUuid)]
#[uuid = "81dd6242-c4cd-4059-a3a0-ed1d0e44e68b"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(default)]
pub struct Camera {
    pub near: f32,
    pub far: f32,
    pub exposure: f32,
    pub projection: Projection,
    pub is_primary: bool
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            near: 0.1,
            far: 1000.0,
            exposure: 1.0,
            projection: Projection::Perspective(45.0),
            is_primary: false
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Projection {
    Perspective(f32), //fov in degrees
    Orthographic,
}

impl Camera {
    pub fn get_projection_matrix(&self, width: f32, height: f32) -> hikari_math::Mat4 {
        match self.projection {
            Projection::Perspective(fov) => hikari_math::Mat4::perspective_rh(
                fov.to_radians(),
                width / height,
                self.near,
                self.far,
            ),
            Projection::Orthographic => hikari_math::Mat4::orthographic_rh(-width, width, -height, height, self.near, self.far),
        }
    }
}
