use hikari_asset::Handle;
use hikari_math::*;
use hikari_render::GpuBuffer;

use crate::{Material, Scene};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tc0: Vec2,
    pub tc1: Vec2,
}
pub struct SubMesh {
    pub vertices: GpuBuffer<Vertex>,
    pub indices: GpuBuffer<u32>,
    pub material: Handle<Material>,
}

pub struct Mesh {
    pub sub_meshes: Vec<SubMesh>,
}

pub fn default_normals(n: usize) -> Vec<Vec3> {
    //Flat normals
    vec![Vec3::ZERO; n]
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MeshSource {
    Scene(Handle<Scene>, usize),
    None,
}
#[derive(Clone, type_uuid::TypeUuid)]
#[uuid = "026f78af-98c8-4c59-9af7-66186eb8d664"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MeshRender {
    pub source: MeshSource,
}
