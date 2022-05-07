use hikari_asset::Handle;
use hikari_asset::MetaData;
use hikari_asset::{Asset, Load};
use hikari_math::*;
use serde::{Deserialize, Serialize};

use crate::texture::Texture2D;

#[derive(Serialize, Deserialize)]
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
    const NAME: &'static str = "Material";

    fn extensions<'a>() -> &'a [&'static str] {
        &["hmat"]
    }
}

impl Load for Material {
    type Loader = ();

    type LoadSettings = ();

    fn load(
        _loader: &Self::Loader,
        data: &[u8],
        _meta: &MetaData<Self>,
        _context: &mut hikari_asset::LoadContext,
    ) -> Result<Self, hikari_asset::Error>
    where
        Self: Sized,
    {
        Ok(serde_yaml::from_slice(data)?)
    }
}
