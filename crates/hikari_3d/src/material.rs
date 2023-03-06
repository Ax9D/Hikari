use hikari_asset::Handle;
use hikari_asset::LoadContext;
use hikari_asset::Saver;
use hikari_asset::{Asset, Loader};
use hikari_math::*;
use serde::{Deserialize, Serialize};

use crate::texture::Texture2D;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Material {
    pub albedo: Option<Handle<Texture2D>>,
    pub uv_set: u32,
    pub albedo_factor: Vec4,
    pub roughness: Option<Handle<Texture2D>>,
    pub roughness_factor: f32,
    pub metallic: Option<Handle<Texture2D>>,
    pub metallic_factor: f32,
    pub emissive: Option<Handle<Texture2D>>,
    pub emissive_strength: f32,
    pub emissive_factor: Vec3,
    pub normal: Option<Handle<Texture2D>>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: None,
            uv_set: 0,
            albedo_factor: Vec4::ONE,
            roughness: None,
            roughness_factor: 1.0,
            metallic: None,
            metallic_factor: 0.0,
            emissive: None,
            emissive_factor: Vec3::ZERO,
            emissive_strength: 1.0,
            normal: None,
        }
    }
}

impl Asset for Material {
    type Settings = ();
}

pub const SUPPORTED_MATERIAL_EXTENSIONS: [&'static str; 1] = ["hmat"];
pub struct MaterialLoader;

impl Loader for MaterialLoader {
    fn load(&self, context: &mut LoadContext) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        let material: Material = serde_yaml::from_reader(context.reader())?;

        material
            .albedo
            .as_ref()
            .map(|texture| context.depends_on(texture));
        material
            .roughness
            .as_ref()
            .map(|texture| context.depends_on(texture));
        material
            .metallic
            .as_ref()
            .map(|texture| context.depends_on(texture));
        material
            .normal
            .as_ref()
            .map(|texture| context.depends_on(texture));
        material.
            emissive
            .as_ref()
            .map(|texture| context.depends_on(texture));

        context.set_asset(material);
        Ok(())
    }

    fn extensions(&self) -> &[&str] {
        &SUPPORTED_MATERIAL_EXTENSIONS
    }
}

impl Saver for MaterialLoader {
    fn extensions(&self) -> &[&str] {
        &SUPPORTED_MATERIAL_EXTENSIONS
    }

    fn save(
        &self,
        context: &mut hikari_asset::SaveContext,
        writer: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        serde_yaml::to_writer(writer, context.get_asset::<Material>())?;

        Ok(())
    }
}
