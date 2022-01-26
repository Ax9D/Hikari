// #[derive(Clone)]
// pub enum MaterialColor {
//     Constant(glam::Vec4),
//     Texture(Arc<Graphy::texture::Texture2D>)
// }
// #[derive(Clone)]
// pub enum MaterialValue {
//     Constant(f32),
//     Texture(Arc<Graphy::texture::Texture2D>)
// }
// #[derive(Clone)]
// pub struct Material {
//     pub(crate)name: String,
//     pub(crate)albedo: MaterialColor,
//     pub(crate)roughness: MaterialValue,
//     pub(crate)metallic: MaterialValue,
// }

// impl Default for Material {
//     fn default() -> Self {
//         Self {
//             name: String::from("Default_Material"),
//             albedo: MaterialColor::Constant(glam::Vec4::ZERO),
//             roughness: MaterialValue::Constant(0.5),
//             metallic: MaterialValue::Constant(0.0)
//         }
//     }
// }
#[derive(Clone, Debug)]
pub struct TextureDesc {
    pub index: usize,
    pub tex_coord_set: u32,
}
#[derive(Clone, Debug)]
pub struct Material {
    pub name: String,
    pub albedo: glam::Vec4,
    pub albedo_map: Option<TextureDesc>,
    pub roughness: f32,
    pub roughness_map: Option<TextureDesc>,
    pub metallic: f32,
    pub metallic_map: Option<TextureDesc>,
    pub normal_map: Option<TextureDesc>,
}
