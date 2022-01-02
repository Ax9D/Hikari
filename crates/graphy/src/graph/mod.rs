mod allocation;
mod command;
mod framebuffer;
mod pass;
mod resources;
mod runtime;
mod storage;

use crate::texture::SampledImage;
use ash::prelude::VkResult;
use ash::vk;

use self::allocation::AllocationData;
use self::pass::graphics;
use self::pass::AnyPass;
use self::runtime::GraphExecutor;

pub use command::CommandBuffer;
pub use resources::*;
pub use storage::Handle;

pub use pass::compute::*;
pub use pass::graphics::*;
pub use pass::AttachmentConfig;
pub use pass::AttachmentKind;
pub use pass::ImageSize;

use std::collections::HashSet;

use thiserror::Error;

use crate::{texture::ImageConfig, Gfx};

#[derive(Error, Debug)]
pub enum GraphCreationError {
    #[error("Pass name: {0:?} appears more than once")]
    DuplicatePassName(String),
    #[error("Only the last pass can output to swapchain")]
    LastPassPresentOnly,
    #[error("Graph resource allocation failed {0}")]
    AllocationFailed(String),
}

pub struct GraphBuilder<'a, S, P, R> {
    gfx: &'a mut Gfx,
    passes: Vec<AnyPass<S, P, R>>,
    resources: GraphResources,
    size: (u32, u32),
}

impl<'a, S, P, R> GraphBuilder<'a, S, P, R> {
    pub fn new(gfx: &'a mut Gfx, width: u32, height: u32) -> Self {
        Self {
            gfx,
            passes: Vec::new(),
            resources: GraphResources::new(),
            size: (width, height),
        }
    }
    pub fn create_image(
        &mut self,
        name: &str,
        config: ImageConfig,
        size: ImageSize,
    ) -> Result<Handle<SampledImage>, GraphCreationError> {
        let (width, height) = size.get_physical_size(self.size);
        let image = SampledImage::with_dimensions(self.gfx.device(), width, height, config)
            .map_err(|err| GraphCreationError::AllocationFailed(err.to_string()))?;

        Ok(self.resources.add_image(name.to_string(), image, size))
    }
    pub fn resources(&self) -> &GraphResources {
        &self.resources
    }
    pub fn add_renderpass(&mut self, pass: Renderpass<S, P, R>) -> &mut Self {
        self.passes.push(AnyPass::Render(pass));

        self
    }

    fn check_duplicate_pass_names(&mut self) -> Result<(), GraphCreationError> {
        let mut names = HashSet::new();

        for pass in &self.passes {
            if names.contains(pass.name()) {
                return Err(GraphCreationError::DuplicatePassName(
                    pass.name().to_string(),
                ));
            } else {
                names.insert(pass.name().to_string());
            }
        }

        Ok(())
    }
    fn check_only_last_pass_presents_to_swapchain(&mut self) -> Result<(), GraphCreationError> {
        let last_pass_ix = self.passes.len() - 1;
        let present_pass_ix = self
            .passes
            .iter()
            .position(|pass| match pass {
                AnyPass::Render(pass) => pass.present_to_swapchain,
                AnyPass::Compute(_) => false,
            })
            .unwrap_or(last_pass_ix);

        if present_pass_ix != last_pass_ix {
            Err(GraphCreationError::LastPassPresentOnly)
        } else {
            Ok(())
        }
    }
    pub fn validate(&mut self) -> Result<(), GraphCreationError> {
        self.check_duplicate_pass_names()?;
        self.check_only_last_pass_presents_to_swapchain()?;

        Ok(())
    }

    // fn get_adjacency_list(passes: &Vec<AnyPass<S, P, R>>) -> HashMap<usize, HashSet<usize>> {
    //     let mut adj_list = HashMap::new();

    //     for (id, pass) in passes.iter().enumerate() {
    //         for output in pass.outputs() {
    //             let matching_pass = passes.iter().enumerate().find(|(id, pass)| {
    //                 pass.inputs()
    //                     .iter()
    //                     .find(|other_input| other_input.erased_handle() == output.erased_handle())
    //                     .is_some()
    //             });

    //             if let Some((other_id, other_pass)) = matching_pass {
    //                 if id != other_id {
    //                     adj_list
    //                         .entry(id)
    //                         .or_insert(HashSet::new())
    //                         .insert(other_id);
    //                 }
    //             }
    //         }
    //     }

    //     adj_list
    // }
    // pub fn flatten_(
    //     pass_id: usize,
    //     visited: &mut HashSet<usize>,
    //     adj_list: &HashMap<usize, HashSet<usize>>,
    //     stack: &mut Vec<usize>,
    // ) {
    //     visited.insert(pass_id);

    //     for node_id in &adj_list[&pass_id] {
    //         if !visited.contains(node_id) {
    //             Self::flatten_(*node_id, visited, adj_list, stack);
    //         }
    //     }

    //     stack.push(pass_id);
    // }
    // pub fn flatten(passes: &Vec<AnyPass<S, P, R>>) -> Vec<usize> {
    //     let mut stack = Vec::new();

    //     let mut visited = HashSet::new();

    //     let mut adj_list = Self::get_adjacency_list(passes);

    //     for (id, pass) in passes.iter().enumerate() {
    //         if !visited.contains(&id) {
    //             Self::flatten_(id, &mut visited, &adj_list, &mut stack);
    //         }
    //     }

    //     stack.reverse();

    //     stack
    // }
    pub fn build(mut self) -> Result<Graph<S, P, R>, GraphCreationError> {
        self.validate()?;

        let allocation_data = AllocationData::new(self.gfx.device(), &self.passes, &self.resources)
            .map_err(|err| GraphCreationError::AllocationFailed(err.to_string()))?;

        let executor = GraphExecutor::new(self.gfx.device()).unwrap();

        Ok(Graph {
            passes: self.passes,
            resources: self.resources,
            allocation_data,
            executor,
            size: self.size,
        })
    }
}

pub struct Graph<S, P, R> {
    passes: Vec<AnyPass<S, P, R>>,
    resources: GraphResources,
    allocation_data: AllocationData,
    executor: GraphExecutor,
    size: (u32, u32),
}

impl<S, P, R> Graph<S, P, R> {
    pub fn execute(
        &mut self,
        gfx: &crate::Gfx,
        scene: &S,
        perframe: &P,
        resources: &R,
    ) -> VkResult<()> {
        self.executor.execute(
            scene,
            perframe,
            resources,
            self.size,
            &mut self.passes,
            &self.resources,
            &self.allocation_data,
            &mut gfx.swapchain().lock(),
        )
    }
    pub fn finish(&mut self) -> VkResult<()> {
        self.executor.finish()
    }
}
