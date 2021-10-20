use std::{sync::Arc};

use ash::{prelude::VkResult, vk};
use fxhash::FxBuildHasher;
use lru::LruCache;

use crate::Shader;

use crate::graph::pass::graphics::pipeline::{PipelineState};
use crate::util::CacheMap;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct PipelineStateVector {
    pub shader: Arc<crate::Shader>,
    pub pipeline_state: PipelineState,
}

pub struct PipelineLookup {
    device: Arc<crate::Device>,
    vk_pipeline_cache: vk::PipelineCache,
    pipelines: CacheMap<PipelineStateVector, vk::Pipeline>
}

impl PipelineLookup {
    fn new(device: &Arc<crate::Device>, capacity: usize) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(
        Self {
            device: device.clone(),
            vk_pipeline_cache: device.pipeline_cache(),
            pipelines: CacheMap::new(capacity),
        })

    }
    fn create_pipeline(
        &self,
        shader: &Shader,
        pipeline_state: &PipelineState,
        vk_renderpass: vk::RenderPass,
        n_color_attachments: usize,
    ) -> VkResult<vk::Pipeline> {
        Ok(unsafe {
            let pipeline = pipeline_state.create_pipeline(&self.device, shader, vk_renderpass, n_color_attachments);
            
            log::debug!("Created new pipeline {:?}", pipeline);

            pipeline
        })
    }
    fn destroy_pipeline(&self, vk_pipeline: vk::Pipeline) {
        unsafe {
            self.device.raw().destroy_pipeline(vk_pipeline, None);
            log::debug!("Destroyed pipeline: {:?}", vk_pipeline);
        }
    }
    pub fn get_vk_pipeline(
        &mut self,
        pipeline_state_vector: &PipelineStateVector,
        renderpass: vk::RenderPass,
        n_color_attachments: usize,
    ) -> VkResult<vk::Pipeline> {

        let pipeline = self.pipelines.get(pipeline_state_vector, |psv| {
            unsafe {
                Ok( psv.pipeline_state.create_pipeline(&self.device, &psv.shader, renderpass, n_color_attachments))
            }
        })?;

        Ok(*pipeline)
    }

    //Call once per frame
    pub fn garbage_collect(&mut self) {
        for pipeline in self.pipelines.unused().drain(..) {
            self.destroy_pipeline(pipeline);
        }
    }
}

impl Drop for PipelineLookup {
    fn drop(&mut self) {
        self.garbage_collect();
        
        unsafe {
            self.device.raw().destroy_pipeline_cache(self.vk_pipeline_cache, None);
        }
    }
}
