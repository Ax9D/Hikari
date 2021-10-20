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
    pub(crate) index: usize,
    pub(crate) tex_coord_set: u32,
}
impl TextureDesc {
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn tex_coord_set(&self) -> u32 {
        self.tex_coord_set
    }
}
#[derive(Clone, Debug)]
pub struct Material {
    pub(crate) name: String,
    pub(crate) albedo: glam::Vec4,
    pub(crate) albedo_map: Option<TextureDesc>,
    pub(crate) roughness: f32,
    pub(crate) roughness_map: Option<TextureDesc>,
    pub(crate) metallic: f32,
    pub(crate) metallic_map: Option<TextureDesc>,
    pub(crate) normal_map: Option<TextureDesc>,
}
impl Material {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn albedo(&self) -> &glam::Vec4 {
        &self.albedo
    }
    pub fn albedo_map(&self) -> Option<&TextureDesc> {
        self.albedo_map.as_ref()
    }
    pub fn roughness(&self) -> f32 {
        self.roughness
    }
    pub fn roughness_map(&self) -> Option<&TextureDesc> {
        self.roughness_map.as_ref()
    }
    pub fn metallic(&self) -> f32 {
        self.metallic
    }
    pub fn metallic_map(&self) -> Option<&TextureDesc> {
        self.metallic_map.as_ref()
    }
    pub fn normal_map(&self) -> Option<&TextureDesc> {
        self.normal_map.as_ref()
    }
}
