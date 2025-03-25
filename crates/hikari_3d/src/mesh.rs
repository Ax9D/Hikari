use hikari_asset::{Handle, AssetPool};
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
    pub position: GpuBuffer<Vec3>,
    pub normals: GpuBuffer<Vec3>,
    pub tc0: GpuBuffer<Vec2>,
    pub tc1: GpuBuffer<Vec2>,
    pub indices: GpuBuffer<u32>,
    pub material: Handle<Material>,
}

pub struct SubMeshNew {
    pub position: GpuBuffer<Vec3>,
    pub normals: GpuBuffer<Vec3>,
    pub tc0: GpuBuffer<Vec2>,
    pub tc1: GpuBuffer<Vec2>,
    pub indices: GpuBuffer<u32>,
}

pub struct Mesh {
    pub sub_meshes: Vec<SubMesh>,
    pub transform: Transform,
}

pub struct MeshNew {
    pub sub_meshes: Vec<SubMeshNew>,
    pub transform: Transform,
}

pub fn default_normals(n: usize) -> Vec<Vec3> {
    //Flat normals
    vec![Vec3::ZERO; n]
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MeshSource {
    Scene(Handle<Scene>, usize),
    #[default]
    None,
}

#[derive(Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaterialTable {
    slots: Vec<Option<Handle<Material>>>
}

impl MaterialTable {
    pub fn new(count: usize) -> Self {
        Self {
            slots: vec![None; count]
        }
    }
    pub fn materials(&self) -> &[Option<Handle<Material>>] {
        &self.slots
    }
    pub fn materials_mut(&mut self) -> &mut [Option<Handle<Material>>] {
        &mut self.slots
    }
    pub fn material(&self, index: usize) -> Option<&Handle<Material>> {
        self.slots.get(index)?.as_ref()
    }
    pub fn material_mut(&mut self, index: usize) -> Option<&mut Handle<Material>> {
        self.slots.get_mut(index)?.as_mut()
    }
}
#[derive(Default, Clone, type_uuid::TypeUuid)]
#[uuid = "026f78af-98c8-4c59-9af7-66186eb8d664"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(default))]
pub struct MeshRender {
    pub source: MeshSource,
    pub material_table: MaterialTable
}

impl MeshRender {
    pub fn get_mesh<'a>(&'a self, scenes: &'a AssetPool<Scene>) -> Option<&'a Mesh> {
        let MeshSource::Scene(handle, mesh_ix) = &self.source else {
            return None;
        };

        let Some(scene) = scenes.get(handle) else {
            return None;
        };

        Some(&scene.meshes[*mesh_ix])
    }
    pub fn get_mesh_and_handle<'a>(&'a self, scenes: &'a AssetPool<Scene>) -> Option<(&'a Mesh, &Handle<Scene>)> {
        let MeshSource::Scene(handle, mesh_ix) = &self.source else {
            return None;
        };

        let Some(scene) = scenes.get(handle) else {
            return None;
        };

        Some((&scene.meshes[*mesh_ix], handle))
    }
}