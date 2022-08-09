use hikari_asset::Handle;
use hikari_asset::LoadContext;
use hikari_asset::{Asset, Loader};
use hikari_math::*;
use serde::{Deserialize, Serialize};

use crate::texture::Texture2D;

#[derive(Serialize, Deserialize, Default)]
pub struct Material {
    pub albedo: Option<Handle<Texture2D>>,
    pub albedo_set: i32,
    pub albedo_factor: Vec4,
    pub roughness: Option<Handle<Texture2D>>,
    pub roughness_set: i32,
    pub roughness_factor: f32,
    pub metallic: Option<Handle<Texture2D>>,
    pub metallic_set: i32,
    pub metallic_factor: f32,
    pub normal: Option<Handle<Texture2D>>,
    pub normal_set: i32,
}

impl Asset for Material {
    type Settings = ();
}
pub struct MaterialLoader;

impl Loader for MaterialLoader {
    fn load(&self, context: &mut LoadContext) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        let material: Material = serde_yaml::from_reader(context.reader())?;
        context.set_asset(material);
        Ok(())
    }

    fn extensions(&self) -> &[&str] {
        &["hmat"]
    }
}
