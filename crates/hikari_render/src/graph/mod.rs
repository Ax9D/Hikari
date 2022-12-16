mod allocation;
mod args;
mod command;
mod framebuffer;
mod pass;
mod resources;
mod runtime;
mod storage;

use crate::Buffer;
use crate::texture::SampledImage;
use ash::prelude::VkResult;
use parking_lot::Mutex;

use self::allocation::AllocationData;
use self::pass::graphics;
use self::pass::AnyPass;
use self::runtime::GraphExecutor;

pub use command::CommandBuffer;
pub use command::RenderpassCommands;

pub use resources::*;
pub use storage::GpuHandle;

pub use pass::compute::*;
pub use pass::graphics::*;
pub use pass::AttachmentConfig;
pub use pass::AttachmentKind;
pub use pass::ImageSize;

use std::collections::HashSet;
use std::sync::Arc;

use thiserror::Error;

use crate::{texture::ImageConfig, Gfx};

pub use args::*;

#[derive(Error, Debug)]
pub enum GraphCreationError {
    #[error("Pass name: {0:?} appears more than once")]
    DuplicatePassName(String),
    #[error("Only the last pass can output to swapchain")]
    LastPassPresentOnly,
    #[error("Graph resource allocation failed {0}")]
    AllocationFailed(String),
}

pub struct GraphBuilder<'a, T: Args> {
    gfx: &'a mut Gfx,
    passes: Vec<AnyPass<T>>,
    resources: GraphResources,
    size: (u32, u32),
}

impl<'a, T: Args> GraphBuilder<'a, T> {
    pub fn new(gfx: &'a mut Gfx, width: u32, height: u32) -> Self {
        Self {
            gfx,
            passes: Vec::new(),
            resources: GraphResources::new(),
            size: (width, height),
        }
    }
    /// Allocates a new image, with the provided config and size
    /// Images are automatically resized when the graph is resized when created with an `ImageSize::Relative(.., ..)`
    /// A unique name must also be provided to the image (used for debugging)
    pub fn create_image(
        &mut self,
        name: &str,
        config: ImageConfig,
        size: ImageSize,
    ) -> Result<GpuHandle<SampledImage>, GraphCreationError> {
        let (width, height, depth) = size.get_physical_size_3d(self.size);
        let image = SampledImage::with_dimensions(self.gfx.device(), width, height, depth, config)
            .map_err(|err| GraphCreationError::AllocationFailed(err.to_string()))?;
        Ok(self.resources.add_image(name.to_string(), image, size))
    }
    pub fn add_buffer<B: Buffer + Send + Sync + 'static>(&mut self, name: &str, buffer: B) -> GpuHandle<B> {
        self.resources.add_buffer(name.to_owned(), buffer)
    }
    pub fn resources(&self) -> &GraphResources {
        &self.resources
    }
    pub fn add_renderpass(&mut self, pass: Renderpass<T>) -> &mut Self {
        self.passes.push(AnyPass::Render(pass));

        self
    }
    pub fn add_computepass(&mut self, pass: ComputePass<T>) -> &mut Self {
        self.passes.push(AnyPass::Compute(pass));

        self
    }
    pub fn initial_size(&mut self) -> (u32, u32) {
        self.size
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
    fn validate(&mut self) -> Result<(), GraphCreationError> {
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
    /// Allocates required resources and returns a Graph
    pub fn build(mut self) -> Result<Graph<T>, GraphCreationError> {
        self.validate()?;

        let allocation_data = AllocationData::new(self.gfx.device(), &self.passes, &self.resources)
            .map_err(|err| GraphCreationError::AllocationFailed(err.to_string()))?;

        let executor = GraphExecutor::new(self.gfx.device()).unwrap();

        let outputs_swapchain = self.passes.iter().any(|pass| match pass {
            AnyPass::Render(pass) => pass.present_to_swapchain,
            AnyPass::Compute(_) => false,
        });

        Ok(Graph {
            device: self.gfx.device().clone(),
            swapchain: self.gfx.swapchain().cloned(),
            passes: self.passes,
            resources: self.resources,
            allocation_data,
            executor,
            size: self.size,
            outputs_swapchain,
        })
    }
}
/// A Graph is a collection of passes (Renderpasses + Compute passes), that execute ensuring proper resource synchronization as defined during Graph creation.
/// A Graph is created using the GraphBuilder, and is immutable, meaning new passes cannot be added after creation.
/// The generic parameter T refer to the data that the Graph is to be provided when executing (usually the game world and render resources)
pub struct Graph<T: Args> {
    device: Arc<crate::Device>,
    swapchain: Option<Arc<Mutex<crate::Swapchain>>>,
    passes: Vec<AnyPass<T>>,
    resources: GraphResources,
    allocation_data: AllocationData,
    executor: GraphExecutor,
    size: (u32, u32),
    outputs_swapchain: bool,
}

impl<T: Args> Graph<T> {
    pub fn execute(&mut self, args: <T::Ref as ByRef>::Item) -> VkResult<()> {
        if self.outputs_swapchain {
            self.executor.execute_and_present(
                args,
                self.size,
                &mut self.passes,
                &self.resources,
                &self.allocation_data,
                &mut self
                    .swapchain
                    .as_ref()
                    .expect("Cannot present in headless mode")
                    .lock(),
            )
        } else {
            self.executor.execute(
                args,
                self.size,
                &mut self.passes,
                &self.resources,
                &self.allocation_data,
            )
        }
    }
    pub fn execute_sync(&mut self, args: <T::Ref as ByRef>::Item) -> VkResult<()> {
        self.execute(args)?;
        self.finish()
    }
    /// Finishes rendering the previous frame
    /// Calling this manually is not recommended as this stalls the GPU
    /// Resources(images, buffers etc.) used during the previous frame should be reusable after calling this
    pub fn finish(&mut self) -> VkResult<()> {
        self.executor.finish()
    }
    pub fn size(&self) -> (u32, u32) {
        self.size
    }
    pub fn resize(
        &mut self,
        new_width: u32,
        new_height: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert!(!(new_width == 0 || new_height == 0));

        self.prepare_exit();
        self.size = (new_width, new_height);
        self.resources
            .resize_images(&self.device, new_width, new_height)?;
        self.allocation_data
            .resize_framebuffers(&self.device, &self.passes, &self.resources)?;

        log::debug!("Resized graph width: {new_width} height: {new_height}");
        Ok(())
    }

    ///Should be called after done using the graph just before its dropped to ensure gpu resources can be safely deallocated
    pub fn prepare_exit(&mut self) {
        unsafe { self.device.raw().device_wait_idle() }.unwrap();
    }

    pub fn resources(&self) -> &GraphResources {
        &self.resources
    }
}
