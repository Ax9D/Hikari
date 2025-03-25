use hikari_3d::{Mesh, SubMesh};
use hikari_math::{Transform};
use vec_map::VecMap;

use crate::common::PerInstanceData;

pub struct InstanceBatch {
    submesh: *const SubMesh,
    count: usize,
    per_instance: Vec<PerInstanceData>,
}
impl InstanceBatch {
    #[inline]
    pub fn submesh(&self) -> &SubMesh {
        unsafe{ &*self.submesh }
    }
    #[inline]    
    pub fn count(&self) -> usize {
        self.count
    }
}

impl std::fmt::Debug for InstanceBatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstanceBatch")
        .field("submesh", &self.submesh)
        .field("count", &self.count)
        .finish()
    }
}

pub struct BatchIter<'a> {
    batches: std::slice::Iter<'a, InstanceBatch>,
    instance_id: usize,
    prev_instance_count: usize,
}

impl<'a> BatchIter<'a> {
    pub fn new(batches: std::slice::Iter<'a, InstanceBatch>) -> Self {
        Self {
            batches,
            instance_id: 0,
            prev_instance_count: 0
        }
    }
}

impl<'a> Iterator for BatchIter<'a> {
    type Item = (usize, &'a InstanceBatch);

    fn next(&mut self) -> Option<Self::Item> {
        let batch = self.batches.next()?;

        self.instance_id += self.prev_instance_count;
        self.prev_instance_count = batch.count();

        Some((self.instance_id, batch))
    }
}

#[derive(Default)]
pub struct MeshInstancer { 
    mesh_to_batch_ix: VecMap<usize>,
    submesh_batches: Vec<InstanceBatch>,
}

impl MeshInstancer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_mesh(&mut self, index: usize, mesh: &Mesh, transform: &Transform) {
        let submesh_count = mesh.sub_meshes.len();

        match self.mesh_to_batch_ix.get(index) {
            Some(&batch_ix) => {
                for batch in &mut self.submesh_batches[batch_ix..batch_ix + submesh_count] {
                    batch.count+= 1;
                    batch.per_instance.push(
                        PerInstanceData {
                            transform: transform.get_matrix() * mesh.transform.get_matrix()
                        }
                    )
                }
            }
            None => {
                let batch_ix = self.submesh_batches.len();

                for submesh in &mesh.sub_meshes {
                    self.submesh_batches.push(InstanceBatch {
                        submesh: submesh as *const _,
                        count: 1,
                        //TODO: Remove Allocation 
                        per_instance: vec![
                            PerInstanceData {
                                transform: transform.get_matrix() * mesh.transform.get_matrix()
                            }
                        ]
                    });
                }
                self.mesh_to_batch_ix.insert(index, batch_ix);
            }
        }
    }
    pub fn write_instance_buffer(&self, buffer: &mut [PerInstanceData]) {
        hikari_dev::profile_function!();

        let mut buffer_iter = buffer.iter_mut();

        for batch in &self.submesh_batches {
            for per_instance in &batch.per_instance {
                if let Some(per_instance_out) = buffer_iter.next() {
                    *per_instance_out = *per_instance;
                }
            }
        }
    }
    pub fn batches(&self) -> BatchIter {
        BatchIter::new(self.submesh_batches.iter())
    }
    pub fn new_frame(&mut self) {
        self.mesh_to_batch_ix.clear();
        self.submesh_batches.clear();
    }
}