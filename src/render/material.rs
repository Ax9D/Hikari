#[derive(Clone)]
pub enum MaterialColor {
    Constant(glam::Vec4),
    Texture(hikari_asset::Asset<graphy::Texture2D>, u32),
}
#[derive(Clone)]
pub enum MaterialValue {
    Constant(f32),
    Texture(hikari_asset::Asset<graphy::Texture2D>, u32),
}
#[derive(Clone)]
pub struct Material {
    pub albedo: MaterialColor,
    pub roughness: MaterialValue,
    pub metallic: MaterialValue,
    pub normal: MaterialValue,
}
impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: MaterialColor::Texture(crate::texture::checkerboard().clone(), 0),
            roughness: MaterialValue::Constant(0.5),
            metallic: MaterialValue::Constant(0.0),
            normal: MaterialValue::Constant(1.0),
        }
    }
}
