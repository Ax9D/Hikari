use hikari_asset::AssetPool;
use hikari_asset::Handle;
use hikari_asset::LoadContext;
use hikari_asset::Saver;
use hikari_asset::{Asset, Loader};
use hikari_math::*;
use serde::{Deserialize, Serialize};

use crate::primitives::Primitives;
use crate::texture::Texture2D;

// pub enum TextureSlot {
//     Albedo = 0,
//     Roughness,
//     Metallic,
//     Normal,
//     Emissive,
//     _Count,
// }

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MaterialBuffer {
    albedo: Vec4,
    emissive: Vec3,
    roughness: f32,
    metallic: f32,
    uv_set: u32,
    albedo_ix: i32,
    emissive_ix: i32,
    roughness_ix: i32,
    metallic_ix: i32,
    normal_ix: i32,
}


// pub struct MaterialNew {
//     textures: Vec<Option<Handle<Texture2D>>>,
//     uv_set: u32,
//     is_dirty: bool,
// }

// impl MaterialNew {
//     pub fn new() -> anyhow::Result<Self> {
//         Ok(Self {
//             textures: vec![None; TextureSlot::_Count as usize],
//             uv_set: 0,
//             is_dirty: false
//         })
//     }
//     pub fn get_texture(&self, slot: TextureSlot) -> Option<&Handle<Texture2D>> {
//         self.textures[slot as usize].as_ref()
//     }
//     pub fn set_texture(&mut self, slot: TextureSlot, texture: Handle<Texture2D>) {
//         self.update_texture(slot, |current| *current = Some(texture));
//     }
//     pub fn update_texture(&mut self, slot: TextureSlot, f: impl FnOnce(&mut Option<Handle<Texture2D>>)) {
//         let texture = &mut self.textures[slot as usize];
//         (f)(texture);

//         self.is_dirty = true;
//     }
//     pub fn is_dirty(&self) -> bool {
//         self.is_dirty
//     }
// }


#[derive(Serialize, Deserialize, type_uuid::TypeUuid)]
#[uuid = "4619d2c4-246f-4dc6-acb1-6d34633f7b71"]
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
    //#[serde(skip)]
    //buffer: RingBuffer<MaterialBuffer>
}

// impl Material {
//     pub fn new(device: &Arc<hikari_render::Device>) -> anyhow::Result<Self> {
//         let buffer = hikari_render::create_uniform_buffer(device, 1)?;
//         Ok(Self {
//             albedo: None,
//             uv_set: 0,
//             albedo_factor: Vec4::ONE,
//             roughness: None,
//             roughness_factor: 1.0,
//             metallic: None,
//             metallic_factor: 0.0,
//             emissive: None,
//             emissive_factor: Vec3::ZERO,
//             emissive_strength: 1.0,
//             normal: None,
//             buffer,
//         })
//     }
//     //Call once per frame ONLY
//     pub fn write_buffer(&mut self, textures: &AssetPool<Texture2D>, primitives: &Primitives) {
//         self.buffer.new_frame();
//         let buffer = MaterialBuffer {
//             albedo: self.albedo_factor,
//             emissive: self.emissive_factor,
//             roughness: self.roughness_factor,
//             metallic: self.metallic_factor,
//             uvSet: self.uv_set,
//             albedoIx: todo!(),
//             emissiveIx: todo!(),
//             roughnessIx: todo!(),
//             metallicIx: todo!(),
//             normalIx: todo!(),
//         };
//         let out_buffer = &mut self.buffer.mapped_slice_mut()[0];

//     }
// }

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
           // buffer,
        }
    }
}

fn resolve_texture_bindless<'a>(
    handle: &Option<Handle<Texture2D>>,
    textures: &'a AssetPool<Texture2D>,
    default: &'a Texture2D,
) -> i32 {
    let index = handle
        .as_ref()
        .map(|handle| {
            let texture = textures.get(handle).unwrap_or(default);
            let bindless_handle = texture.raw().bindless_handle(0);
            bindless_handle.index() as i32
        })
        .unwrap_or(-1);

    index
}
impl Material {
    pub fn prepare_render(&self, buffer: &mut MaterialBuffer, textures: &AssetPool<Texture2D>, primitives: &Primitives) {
        buffer.albedo = self.albedo_factor;
        buffer.roughness = self.roughness_factor;
        buffer.metallic = self.metallic_factor;
        buffer.emissive = self.emissive_factor * self.emissive_strength;

        buffer.albedo_ix = resolve_texture_bindless(&self.albedo, textures, &primitives.checkerboard);
        buffer.roughness_ix = resolve_texture_bindless(&self.roughness, textures, &primitives.black);
        buffer.metallic_ix = resolve_texture_bindless(&self.metallic, textures, &primitives.black);
        buffer.normal_ix = resolve_texture_bindless(&self.normal, textures, &primitives.black);
        buffer.emissive_ix = resolve_texture_bindless(&self.emissive, textures, &primitives.black);
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
        material
            .emissive
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
