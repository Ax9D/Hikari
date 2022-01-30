use crate::core::primitives::Transform;
pub mod camera;
pub mod light;
pub mod material;
pub mod model;
pub mod scenerenderer;
pub mod texture;

pub use camera::CameraComponent;
pub use light::DirectionalLight;
pub use material::Material;
pub use model::Mesh;
pub use model::Model;

const QUAD_VERTS: [f32; 2 * 4] = [-1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0];
const QUAD_INDS: [i32; 6] = [0, 1, 2, 0, 2, 3];
const QUAD_TEX_COORDS: [f32; 2 * 4] = [0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0];
