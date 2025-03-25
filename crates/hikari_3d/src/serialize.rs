use std::sync::Arc;

use hikari_math::{Vec2, Vec3, Transform};
use hikari_render::{GpuBuffer, Device, vk};
use rkyv::Archived;

use crate::{SubMeshNew, MeshNew};

const MAGIC_NUM: &[u8; 8] = b"hkmesh\0\0";

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SubMeshFile {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub tc0: Vec<Vec2>,
    pub tc1: Vec<Vec2>,
    pub indices: Vec<u32>,
}
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct MeshFile {
    _magic: [u8; 8],
    pub transform: Transform,
    pub sub_meshes: Vec<SubMeshFile>,
}

impl MeshNew {
    pub fn from_archive(device: &Arc<Device>, archive: &Archived<MeshFile>) -> anyhow::Result<Self> {
        let mut sub_meshes = Vec::new();
        for submesh in archive.sub_meshes.iter() {
            sub_meshes.push(SubMeshNew::from_archive(device, submesh)?);
        }
        let transform = Transform {
            position: archive.transform.position,
            scale: archive.transform.scale,
            rotation: archive.transform.rotation
        };

        Ok(Self {
            sub_meshes,
            transform
        })
    }
    pub fn to_file(&self) -> anyhow::Result<MeshFile> {
        let transform = self.transform;
        let mut submesh_files = Vec::new();

        for submesh in &self.sub_meshes {
            let positions = submesh.position.download(0..submesh.position.capacity())?;
            let positions = positions.mapped_slice().to_owned();

            let normals = submesh.normals.download(0..submesh.normals.capacity())?;
            let normals = normals.mapped_slice().to_owned();

            let tc0 = submesh.tc0.download(0..submesh.tc0.capacity())?;
            let tc0 = tc0.mapped_slice().to_owned();

            let tc1 = submesh.tc1.download(0..submesh.tc1.capacity())?;
            let tc1 = tc1.mapped_slice().to_owned();

            let indices = submesh.indices.download(0..submesh.indices.capacity())?;
            let indices = indices.mapped_slice().to_owned();

            submesh_files.push(
                SubMeshFile {
                    positions,
                    normals,
                    tc0,
                    tc1,
                    indices,
                }
            );
        }
        
        Ok(MeshFile {
            _magic: *MAGIC_NUM,
            transform,
            sub_meshes: submesh_files
        })
    }
}

impl SubMeshNew {
    pub fn from_archive(device: &Arc<Device>, archive: &Archived<SubMeshFile>) -> anyhow::Result<Self> {
        let mut position = GpuBuffer::new(device, archive.positions.len(), vk::BufferUsageFlags::VERTEX_BUFFER)?;
        let mut normals = GpuBuffer::new(device, archive.normals.len(), vk::BufferUsageFlags::VERTEX_BUFFER)?;
        let mut tc0 =  GpuBuffer::new(device, archive.tc0.len(), vk::BufferUsageFlags::VERTEX_BUFFER)?;
        let mut tc1 =  GpuBuffer::new(device, archive.tc1.len(), vk::BufferUsageFlags::VERTEX_BUFFER)?;
        let mut indices = GpuBuffer::new(device, archive.indices.len(), vk::BufferUsageFlags::INDEX_BUFFER)?;

        position.upload(&archive.positions, 0)?;
        normals.upload(&archive.normals, 0)?;
        tc0.upload(&archive.tc0, 0)?;
        tc1.upload(&archive.tc1, 0)?;
        indices.upload(&archive.indices, 0)?;

        Ok(Self {
            position,
            normals,
            tc0,
            tc1,
            indices
        })
    }
}