mod command;
mod framebuffer;
mod pass;
mod pipeline;
mod storage;

pub use command::CommandBuffer;
use vec_map::VecMap;

use std::collections::HashSet;
use std::sync::Arc;

use crate::descriptor::DescriptorPool;
use crate::graph::command::CommandBufferSavedState;
use crate::texture::SampledImage;

use ash::prelude::VkResult;
use ash::vk;

pub use pass::graphics;
pub use pass::ColorFormat;
pub use pass::DepthStencilFormat;
pub use pass::ImageSize;

//use self::renderpass::PassSpecific;

use pass::*;

use thiserror::Error;

use std::collections::HashMap;

use self::pass::graphics::Renderpass;
use self::pipeline::PipelineLookup;

#[derive(Error, Debug)]
pub enum GraphCompilationError {
    #[error("Atleast one terminal pass is required, None provided.")]
    NoTerminalPass,

    // #[error("Only one window color output is permitted.")]
    // MultipleSwapchainColorOutputs,

    // #[error("Only one window depth output is permitted. ")]
    // MultipleSwapchainDepthOutputs,
    #[error("Only one swapchain output is permitted per graph. ")]
    MultipleSwapchainOutputs,

    #[error("Pass name: {0:?} appears more than once")]
    DuplicatePassName(String),

    #[error(
        "Duplicate output in renderpass: {0:?}, output
         {1:?} already appears in renderpass {2:?}"
    )]
    DuplicateOutput(String, String, String),

    #[error("In renderpass: {0:?}, Input {1:?} doesn't have a corresponding output!")]
    UnknownInput(String, String),

    #[error("Cyclic Dependency detected, Look at renderpass: {0:?}")]
    CyclicDependency(String),
}

pub struct Graph<Scene, PerFrame, Resources> {
    graph: ProcessedGraph<Scene, PerFrame, Resources>,
    descriptor_pool: DescriptorPool,
    pipeline_lookup: PipelineLookup,
    pass_resources: Vec<PassResources>,
    allocation_data: AllocationData,
    flat: Vec<usize>,
    size: (u32, u32),
}
impl<Scene, PerFrame, Resources> Graph<Scene, PerFrame, Resources> {
    pub fn execute(
        &mut self,
        gfx: &mut crate::Gfx,
        scene: &Scene,
        perframe: &PerFrame,
        resources: &Resources,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let last_frame = gfx.frame_state().last_frame();
        unsafe {
            let fences = &[last_frame.render_finished_fence];
            log::debug!("Fetched render finished fence");
            gfx.device()
                .raw()
                .wait_for_fences(fences, true, 1000000000)?;

            gfx.device()
                .raw()
                .reset_fences(&[last_frame.render_finished_fence])?;
        }

        log::debug!("Reset fences");
        let current_frame = gfx.frame_state().current_frame();

        let mut command_buffer = CommandBuffer::from_existing(
            gfx.device(),
            current_frame.command_buffer,
            CommandBufferSavedState {
                pipeline_lookup: &mut self.pipeline_lookup,
                descriptor_pool: &mut self.descriptor_pool,
            },
        );

        command_buffer.reset();

        for &pass_ix in &self.flat {
            let node = &self.graph.nodes[pass_ix];
            let pass_resources = &self.pass_resources[pass_ix];
            match &pass_resources.renderpass {
                Some(renderpass) => {


                    let image_size = match node.node_data {
                        NodeData::Graphics(size, _) => size,
                        _ => unreachable!(),
                    };

                    let (width, height) = image_size.get_size(self.size);
                    
                    let create_info = vk::RenderPassBeginInfo::builder()
                    .render_pass(renderpass.inner)
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: vk::Extent2D { width, height },
                    });
                    //.framebuffer();
                    
                    command_buffer.begin_renderpass(renderpass.clone(), &create_info);
                    
                    Self::bind_resources(
                        &self.allocation_data,
                        &mut command_buffer,
                        &pass_resources.resource_bind_infos,
                    );
                    
                    log::debug!("Here");
                    (node.run_fn)(&mut command_buffer, scene, perframe, resources);

                    unsafe {
                        self.allocation_data.get_barrier_storage(pass_ix)
                        .expect("Internal implementation error, something has gone horribly wrong...")
                        .apply(gfx.device(), command_buffer.raw());
                    }

                    command_buffer.end_renderpass();

                    if node.outputs_swapchain() {}
                }
                None => todo!(),
            }

            //begin renderpass
            //
        }

        gfx.frame_state_mut().update();

        log::debug!("Updated frame state");

        self.descriptor_pool.new_frame();
        log::debug!("Updated descriptor pool");

        self.pipeline_lookup.new_frame();
        log::debug!("Updated pipeline lookup");

        Ok(())
    }

    fn bind_resources(
        allocation_data: &AllocationData,
        cmd: &mut CommandBuffer,
        bind_infos: &[ResourceBindInfo],
    ) {
        for bind_info in bind_infos {
            let image = allocation_data
            .get_image(bind_info.image_ix)
                .expect("Internal implementation error");

            cmd.set_image(image, 0, bind_info.binding);
        }
    }
}

struct ProcessedInput {
    name: String,
    parent_pass_ix: usize,
    input: Input,
}
struct ProcessedOutput {
    name: String,
    parent_node_ix: usize,
    output: Output,
}

///Collection of barriers for each node
pub(super) struct BarrierStorage {
    image_barriers: Vec<vk::ImageMemoryBarrier2KHR>,
}
impl BarrierStorage {
    pub fn new(image_barriers: Vec<vk::ImageMemoryBarrier2KHR>) -> Self {
        Self { image_barriers }
    }
    pub unsafe fn apply(&self, device: &Arc<crate::Device>, cmd: vk::CommandBuffer) {
        let dependency_info = vk::DependencyInfoKHR::builder()
            .image_memory_barriers(&self.image_barriers)
            .dependency_flags(vk::DependencyFlags::BY_REGION);

        device
            .extensions()
            .synchronization2
            .cmd_pipeline_barrier2(cmd, &dependency_info);
    }
}

struct ProcessedGraph<S, P, R> {
    all_inputs: Vec<ProcessedInput>,
    all_outputs: Vec<ProcessedOutput>,
    nodes: Vec<ProcessedNode<S, P, R>>,
    input_to_output: HashMap<usize, usize>, //Input to dependency/output
    output_to_inputs: HashMap<usize, Vec<usize>>, // Output to input(s)

    outputs_swapchain: bool,
}
enum NodeData {
    Graphics(ImageSize, bool),
    Compute,
}
struct ProcessedNode<Scene, PerFrame, Resources> {
    pub name: String,
    pub inputs: Vec<usize>,  //Indices
    pub outputs: Vec<usize>, //Indices to image array,
    pub run_fn: Box<dyn Fn(&mut CommandBuffer, &Scene, &PerFrame, &Resources)>,
    pub is_final: bool,
    pub node_data: NodeData,
}
impl<Scene, PerFrame, Resources> ProcessedNode<Scene, PerFrame, Resources> {
    #[inline]
    pub fn outputs_swapchain(&self) -> bool {
        match self.node_data {
            NodeData::Graphics(_, output_swapchain) => output_swapchain,
            NodeData::Compute => false,
        }
    }
}
pub(super) struct AllocationData {
    device: Arc<crate::Device>,
    images: VecMap<SampledImage>,          //Output ix to image
    framebuffers: VecMap<vk::Framebuffer>, //Node ix to framebuffer
    barriers: VecMap<BarrierStorage>,      //buffers: ...
}

impl AllocationData {
    pub fn new(device: &Arc<crate::Device>) -> Self {
        Self {
            device: device.clone(),
            images: VecMap::new(),
            framebuffers: VecMap::new(),
            barriers: VecMap::new(),
        }
    }
    pub fn add_image(&mut self, output_ix: usize, image: SampledImage) {
        if let Some(_) = self.images.insert(output_ix, image) {
            panic!("Image with same index already exists")
        }
    }
    pub fn get_image(&self, output_ix: usize) -> Option<&SampledImage> {
        self.images.get(output_ix)
    }
    pub fn add_framebuffer(&mut self, node_ix: usize, framebuffer: vk::Framebuffer) {
        if let Some(_) = self.framebuffers.insert(node_ix, framebuffer) {
            panic!("Framebuffer with same index already exists");
        }
    }
    pub fn get_framebuffer(&self, node_ix: usize) -> Option<&vk::Framebuffer> {
        self.framebuffers.get(node_ix)
    }
    pub fn add_barrier_storage(&mut self, node_ix: usize, barrier_storage: BarrierStorage) {
        if let Some(_) = self.barriers.insert(node_ix, barrier_storage) {
            panic!("Barrier with same index already exists");
        }
    }
    pub fn get_barrier_storage(&mut self, node_ix: usize) -> Option<&BarrierStorage> {
        self.barriers.get(node_ix)
    }
}
impl Drop for AllocationData {
    fn drop(&mut self) {
        for &framebuffer in self.framebuffers.values() {
            unsafe {
                self.device.raw().destroy_framebuffer(framebuffer, None);
            }
        }
    }
}
enum Resource {
    Image(vk::ImageView),
}
pub(super) struct ResourceBindInfo {
    image_ix: usize,
    binding: u32, //of set 0
}
#[derive(Clone)]
pub(super) struct CompiledRenderpass {
    pub inner: vk::RenderPass,
    pub n_color_attachments: usize,
}
struct PassResources {
    resource_bind_infos: Vec<ResourceBindInfo>,
    renderpass: Option<CompiledRenderpass>,
}
impl PassResources {
    fn delete(self, device: &Arc<crate::Device>) {
        unsafe {
            if let Some(pass) = self.renderpass {
                device.raw().destroy_render_pass(pass.inner, None);
            }
        }
    }
}

pub struct GraphBuilder<Scene, PerFrame, Resources> {
    size: Option<(u32, u32)>,
    passes: Vec<Renderpass<Scene, PerFrame, Resources>>,
}

impl<Scene, PerFrame, Resources> GraphBuilder<Scene, PerFrame, Resources> {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            size: None,
        }
    }
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = Some((width, height));

        self
    }
    pub fn add_renderpass(mut self, renderpass: Renderpass<Scene, PerFrame, Resources>) -> Self {
        self.passes.push(renderpass);

        self
    }
    fn generate_processed_graph(self) -> ProcessedGraph<Scene, PerFrame, Resources> {
        let mut all_inputs = Vec::new();
        let mut all_outputs = Vec::new();
        let mut nodes = Vec::new();

        for pass in self.passes {
            let mut inputs = Vec::new();
            let mut outputs = Vec::new();

            for (name, input) in pass.inputs() {
                inputs.push(all_inputs.len());

                all_inputs.push(ProcessedInput {
                    name: name.clone(),
                    parent_pass_ix: nodes.len(),
                    input: input.clone(),
                });
            }

            for (name, output) in pass.outputs() {
                outputs.push(all_outputs.len());

                all_outputs.push(ProcessedOutput {
                    name: name.clone(),
                    parent_node_ix: nodes.len(),
                    output: output.clone(),
                });
            }
            let size = pass.size().clone();
            let node = ProcessedNode {
                name: pass.name().to_string(),
                inputs,
                outputs,
                is_final: pass.is_final(),
                node_data: NodeData::Graphics(size, pass.outputs_swapchain()),
                run_fn: pass.draw_fn,
            };

            nodes.push(node);
        }

        let mut backward_edges = HashMap::new();
        let mut forward_edges = HashMap::new();

        for node in &nodes {
            for &input_ix in &node.inputs {
                let input = &all_inputs[input_ix];
                let dependency_output_ix = all_outputs
                    .iter()
                    .position(|output| output.name == input.name);

                if let Some(dependency_output_ix) = dependency_output_ix {
                    backward_edges.insert(input_ix, dependency_output_ix);
                }
            }

            for &output_ix in &node.outputs {
                let output = &all_outputs[output_ix];
                let connected_input_ixs: Vec<_> = all_inputs
                    .iter()
                    .enumerate()
                    .filter_map(|(ix, input)| {
                        if input.name == output.name {
                            Some(ix)
                        } else {
                            None
                        }
                    })
                    .collect();

                forward_edges.insert(output_ix, connected_input_ixs);
            }
        }

        let outputs_swapchain = nodes.iter().any(|node| node.outputs_swapchain());

        ProcessedGraph {
            all_inputs,
            all_outputs,
            nodes,
            input_to_output: backward_edges,
            output_to_inputs: forward_edges,
            outputs_swapchain,
        }
    }
    fn check_duplicate_node_names(
        graph: &ProcessedGraph<Scene, PerFrame, Resources>,
    ) -> Result<(), GraphCompilationError> {
        let mut unique_pass_names = HashSet::new();
        for node in &graph.nodes {
            if unique_pass_names.contains(&node.name) {
                return Err(GraphCompilationError::DuplicatePassName(
                    node.name.to_string(),
                ));
            } else {
                unique_pass_names.insert(node.name.clone());
            }
        }

        Ok(())
    }
    fn check_duplicate_output_names(
        graph: &ProcessedGraph<Scene, PerFrame, Resources>,
    ) -> Result<(), GraphCompilationError> {
        let mut output_to_pass_name: HashMap<String, String> = HashMap::new();

        for node in &graph.nodes {
            for &output_ix in &node.outputs {
                let output_name = &graph.all_outputs[output_ix].name;
                if let Some(other_pass) = output_to_pass_name.get(output_name) {
                    return Err(GraphCompilationError::DuplicateOutput(
                        node.name.clone(),
                        output_name.clone(),
                        other_pass.clone(),
                    ));
                } else {
                    output_to_pass_name.insert(output_name.clone(), node.name.clone());
                }
            }
        }

        let num_swapchain_outputs = graph.nodes.iter().fold(0, |acc, node| {
            if node.outputs_swapchain() {
                acc + 1
            } else {
                acc
            }
        });

        if num_swapchain_outputs > 1 {
            return Err(GraphCompilationError::MultipleSwapchainOutputs);
        }

        Ok(())
    }
    fn check_unknown_inputs(
        graph: &ProcessedGraph<Scene, PerFrame, Resources>,
    ) -> Result<(), GraphCompilationError> {
        for input in &graph.all_inputs {
            log::debug!("{}", input.name);
            if graph
                .all_outputs
                .iter()
                .find(|&output| input.name == output.name)
                .is_none()
            {
                return Err(GraphCompilationError::UnknownInput(
                    graph.nodes[input.parent_pass_ix].name.clone(),
                    input.name.clone(),
                ));
            }
        }

        Ok(())
    }
    fn check_multiple_final_nodes(
        graph: &ProcessedGraph<Scene, PerFrame, Resources>,
    ) -> Result<(), GraphCompilationError> {
        let mut final_node_count = 0;
        for node in &graph.nodes {
            if node.is_final {
                final_node_count += 1;
            }
        }

        if final_node_count > 0 {
            return Ok(());
        }

        return Err(GraphCompilationError::NoTerminalPass);
    }
    fn validate(
        graph: &ProcessedGraph<Scene, PerFrame, Resources>,
    ) -> Result<(), GraphCompilationError> {
        Self::check_duplicate_node_names(&graph)?;
        Self::check_unknown_inputs(&graph)?;
        Self::check_duplicate_output_names(&graph)?;
        Self::check_multiple_final_nodes(&graph)?;

        Ok(())
    }
    fn flatten_(
        to_visit_ix: usize,
        flat: &mut Vec<usize>,
        visited_passes_ix: &mut HashSet<usize>,
        graph: &ProcessedGraph<Scene, PerFrame, Resources>,
    ) -> Result<(), GraphCompilationError> {
        for &input_ix in &graph.nodes[to_visit_ix].inputs {
            let dependency_output_ix = graph.input_to_output[&input_ix];
            let dependency_output = &graph.all_outputs[dependency_output_ix];

            let dependency_pass_ix = dependency_output.parent_node_ix;

            if visited_passes_ix.contains(&dependency_pass_ix) {
                return Err(GraphCompilationError::CyclicDependency(
                    graph.nodes[dependency_pass_ix].name.clone(),
                ));
            }

            log::debug!(
                "Visiting {} through {:?} in {}",
                graph.nodes[dependency_pass_ix].name,
                graph.all_inputs[input_ix].name,
                graph.nodes[to_visit_ix].name
            );

            visited_passes_ix.insert(dependency_pass_ix);
            Self::flatten_(dependency_pass_ix, flat, visited_passes_ix, graph)?;
        }
        flat.push(to_visit_ix);

        Ok(())
    }
    //Remove recursion maybe?
    fn flatten(
        graph: &ProcessedGraph<Scene, PerFrame, Resources>,
    ) -> Result<Vec<usize>, GraphCompilationError> {
        let to_visit_ix = graph.nodes.iter().position(|node| node.is_final).unwrap();
        let mut visited_passes_ix = HashSet::new();
        let mut flat = Vec::new();

        Self::flatten_(to_visit_ix, &mut flat, &mut visited_passes_ix, graph)?;

        Ok(flat)
    }
    fn allocate_images(
        device: &Arc<crate::Device>,
        swapchain: &crate::Swapchain,
        allocation_data: &mut AllocationData,
        graph_size: (u32, u32),
        graph: &ProcessedGraph<Scene, PerFrame, Resources>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let swapchain_size = swapchain.size();

        for (ix, output) in graph.all_outputs.iter().enumerate() {
            let parent_node = &graph.nodes[output.parent_node_ix];

            let mut config = crate::texture::VkTextureConfig {
                format: vk::Format::UNDEFINED, //Temporary
                filtering: vk::Filter::LINEAR,
                wrap_x: vk::SamplerAddressMode::REPEAT,
                wrap_y: vk::SamplerAddressMode::REPEAT,
                aniso_level: 0,
                mip_levels: 1,
                mip_filtering: vk::SamplerMipmapMode::LINEAR,
                aspect_flags: vk::ImageAspectFlags::empty(), //Temporary,
                primary_image_layout: vk::ImageLayout::ATTACHMENT_OPTIMAL_KHR, //Temporary,
                usage: vk::ImageUsageFlags::SAMPLED,
                host_readable: true,
            };

            match &output.output {
                Output::Color(output) => {
                    let size = match &parent_node.node_data {
                        NodeData::Graphics(size, _) => size,
                        NodeData::Compute => unreachable!(),
                    };

                    let (width, height) = size.get_size(graph_size);
                    config.format = output.format.into();
                    config.aspect_flags = vk::ImageAspectFlags::COLOR;
                    config.primary_image_layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
                    config.usage |= vk::ImageUsageFlags::COLOR_ATTACHMENT;

                    allocation_data.add_image(
                        ix,
                        crate::texture::SampledImage::with_dimensions(
                            device, width, height, config,
                        )?,
                    );
                }
                Output::DepthStencil(output) => {
                    let size = match &parent_node.node_data {
                        NodeData::Graphics(size, _) => size,
                        NodeData::Compute => unreachable!(),
                    };

                    let (width, height) = size.get_size(graph_size);
                    config.format = output.format.into();
                    config.aspect_flags = vk::ImageAspectFlags::DEPTH;
                    config.usage |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;

                    if output.format.is_stencil_format() {
                        config.aspect_flags |= vk::ImageAspectFlags::STENCIL;
                        config.primary_image_layout =
                            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
                    } else {
                        config.primary_image_layout = vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL;
                    }

                    allocation_data.add_image(
                        ix,
                        crate::texture::SampledImage::with_dimensions(
                            device, width, height, config,
                        )?,
                    );
                }
                _ => {}
            };
        }

        Ok(())
    }
    fn allocate_framebuffers(
        device: &Arc<crate::Device>,
        swapchain: &crate::Swapchain,
        allocation_data: &mut AllocationData,
        pass_resources: &[PassResources],
        graph_size: (u32, u32),
        graph: &ProcessedGraph<Scene, PerFrame, Resources>,
    ) -> VkResult<()> {
        for (ix, (node, resources)) in graph.nodes.iter().zip(pass_resources.iter()).enumerate() {
            if let Some(CompiledRenderpass { inner, .. }) = resources.renderpass {
                if node.outputs_swapchain() {
                    let mut attachment_ixs = Vec::new();

                    for &output_ix in &node.outputs {
                        let output = &graph.all_outputs[output_ix];
                        match output.output {
                            Output::Color(_) | Output::DepthStencil(_) => {
                                attachment_ixs.push(output_ix);
                            }
                            _ => {}
                        }
                    }
                    let framebuffer = framebuffer::from_allocation_data(
                        device,
                        allocation_data,
                        &attachment_ixs,
                        inner,
                    )?;

                    allocation_data.add_framebuffer(ix, framebuffer);
                }
            }
        }

        Ok(())
    }
    fn allocate_barriers(
        allocation_data: &mut AllocationData,
        graph: &ProcessedGraph<Scene, PerFrame, Resources>,
    ) {
        for (ix, node) in graph.nodes.iter().enumerate() {
            let mut image_barriers = Vec::new();
            for &output_ix in &node.outputs {
                let output = &graph.all_outputs[output_ix];

                let dst_stage_mask = graph.output_to_inputs[&output_ix].iter().fold(
                    vk::PipelineStageFlags2KHR::empty(),
                    |acc, &input_ix| {
                        let parent_pass_ix = graph.all_inputs[input_ix].parent_pass_ix;
                        let parent_pass = &graph.nodes[parent_pass_ix];

                        match node.node_data {
                            NodeData::Graphics(_, _) => {
                                acc | vk::PipelineStageFlags2KHR::FRAGMENT_SHADER
                            }
                            NodeData::Compute => acc | vk::PipelineStageFlags2KHR::COMPUTE_SHADER,
                        }
                    },
                );

                if dst_stage_mask.is_empty() {
                    log::warn!(
                        "Output {:?} of pass {:?} is unused",
                        output.name,
                        graph.nodes[output.parent_node_ix].name
                    );
                    continue;
                }

                let src_stage_mask;
                let src_access_mask;

                let dst_access_mask;
                let old_layout;
                let new_layout;

                match output.output {
                    Output::Color(_) => {
                        src_stage_mask = vk::PipelineStageFlags2KHR::COLOR_ATTACHMENT_OUTPUT;
                        //dst_stage_mask = vk::PipelineStageFlags2KHR::FRAGMENT_SHADER;

                        src_access_mask = vk::AccessFlags2KHR::COLOR_ATTACHMENT_WRITE;
                        dst_access_mask = vk::AccessFlags2KHR::SHADER_READ;

                        old_layout = vk::ImageLayout::ATTACHMENT_OPTIMAL_KHR;
                        new_layout = vk::ImageLayout::READ_ONLY_OPTIMAL_KHR;
                    }
                    Output::DepthStencil(_) => {
                        src_stage_mask = vk::PipelineStageFlags2KHR::EARLY_FRAGMENT_TESTS
                            | vk::PipelineStageFlags2KHR::LATE_FRAGMENT_TESTS;
                        //dst_stage_mask = vk::PipelineStageFlags2KHR::FRAGMENT_SHADER;

                        src_access_mask = vk::AccessFlags2KHR::DEPTH_STENCIL_ATTACHMENT_WRITE;
                        dst_access_mask = vk::AccessFlags2KHR::SHADER_READ;

                        old_layout = vk::ImageLayout::ATTACHMENT_OPTIMAL_KHR;
                        new_layout = vk::ImageLayout::READ_ONLY_OPTIMAL_KHR;
                    }
                    Output::StorageBuffer => todo!(),
                }

                image_barriers.push(
                    *vk::ImageMemoryBarrier2KHR::builder()
                        .src_access_mask(src_access_mask)
                        .dst_access_mask(dst_access_mask)
                        .src_stage_mask(src_stage_mask)
                        .dst_stage_mask(dst_stage_mask)
                        .old_layout(old_layout)
                        .new_layout(new_layout)
                        .image(allocation_data.get_image(output_ix).unwrap().image()),
                );
            }

            let barrier_storage = BarrierStorage::new(image_barriers);

            allocation_data.add_barrier_storage(ix, barrier_storage);
        }
    }
    fn allocate_pass_resources(
        device: &Arc<crate::Device>,
        swapchain: &crate::Swapchain,
        graph: &ProcessedGraph<Scene, PerFrame, Resources>,
    ) -> Result<Vec<PassResources>, Box<dyn std::error::Error>> {
        let mut resources = Vec::new();
        for (node_ix, _) in graph.nodes.iter().enumerate() {
            resources.push(Self::allocate_single_pass_resources(
                device, swapchain, node_ix, graph,
            )?)
        }

        Ok(resources)
    }
    fn create_render_pass(
        device: &Arc<crate::Device>,
        all_outputs: &[ProcessedOutput],
        output_ixs: &[usize],
    ) -> VkResult<(vk::RenderPass, usize)> {
        let mut attachments = Vec::new();
        let mut color_attachment_refs = Vec::new();
        let mut depth_attachment_ref = None;

        for &output_ix in output_ixs {
            let output = &all_outputs[output_ix];

            if output.output.is_graphics() {
                let load_op;
                let store_op = vk::AttachmentStoreOp::STORE;
                let stencil_load_op;
                let stencil_store_op = vk::AttachmentStoreOp::STORE;

                let format;
                let final_layout;

                let layout;

                match &output.output {
                    Output::Color(output) => {
                        load_op = if output.clear {
                            vk::AttachmentLoadOp::CLEAR
                        } else {
                            vk::AttachmentLoadOp::LOAD
                        };
                        layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;

                        final_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;

                        stencil_load_op = vk::AttachmentLoadOp::DONT_CARE;

                        format = output.format.clone().into();
                    }
                    Output::DepthStencil(output) => {
                        load_op = if output.depth_clear {
                            vk::AttachmentLoadOp::CLEAR
                        } else {
                            vk::AttachmentLoadOp::LOAD
                        };

                        stencil_load_op = if output.stencil_clear {
                            vk::AttachmentLoadOp::CLEAR
                        } else {
                            vk::AttachmentLoadOp::LOAD
                        };

                        layout = vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;

                        final_layout = vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL;
                        format = output.format.clone().into();
                    }

                    Output::StorageBuffer => unreachable!(),
                }

                log::info!("{:?} {:?}", layout, final_layout);

                let attachment = *vk::AttachmentDescription::builder()
                    .format(format)
                    .load_op(load_op)
                    .store_op(store_op)
                    .stencil_store_op(stencil_store_op)
                    .stencil_load_op(stencil_load_op)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(final_layout);

                if layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL {
                    color_attachment_refs.push(
                        *vk::AttachmentReference::builder()
                            .attachment(attachments.len() as u32)
                            .layout(layout),
                    );
                } else {
                    depth_attachment_ref.replace(
                        *vk::AttachmentReference::builder()
                            .attachment(attachments.len() as u32)
                            .layout(layout),
                    );
                }

                attachments.push(attachment);
            }
        }
        let mut subpass_desc = *vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs);

        if let Some(depth_stencil_attachment_ref) = &depth_attachment_ref {
            subpass_desc.p_depth_stencil_attachment = depth_stencil_attachment_ref as *const _;
        }

        let subpass_descs = [subpass_desc];
        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpass_descs);

        let pass = unsafe { device.raw().create_render_pass(&create_info, None)? };

        log::debug!("Created renderpass");

        Ok((pass, color_attachment_refs.len()))
    }
    fn allocate_single_pass_resources(
        device: &Arc<crate::Device>,
        swapchain: &crate::Swapchain,
        node_ix: usize,
        graph: &ProcessedGraph<Scene, PerFrame, Resources>,
    ) -> Result<PassResources, Box<dyn std::error::Error>> {
        let mut image_bind_infos = Vec::new();

        let node = &graph.nodes[node_ix];

        for input_ix in &node.inputs {
            let input = &graph.all_inputs[*input_ix];
            match input.input {
                Input::Dependency => {}

                Input::Read(binding) => {
                    let dependency_output_ix = graph.input_to_output[input_ix];
                    let dependency_output = &graph.all_outputs[dependency_output_ix];

                    match dependency_output.output {
                        Output::Color(_) | Output::DepthStencil(_) => {
                            image_bind_infos.push(ResourceBindInfo {
                                image_ix: dependency_output_ix,
                                binding,
                            });
                        }

                        Output::StorageBuffer => {
                            todo!()
                        }
                    }
                }
            }
        }

        let renderpass = match &node.node_data {
            NodeData::Graphics(size, outputs_swapchain) => {
                let (vk_renderpass, n_color_attachments) = if *outputs_swapchain {
                    (swapchain.renderpass(), 1)
                } else {
                    Self::create_render_pass(device, &graph.all_outputs, &node.outputs)?
                };
                Some(CompiledRenderpass {
                    inner: vk_renderpass,
                    n_color_attachments,
                })
            }

            NodeData::Compute => None,
        };

        let resources = PassResources {
            resource_bind_infos: image_bind_infos,
            renderpass,
        };

        Ok(resources)
    }
    fn compile(
        device: &Arc<crate::Device>,
        swapchain: &crate::Swapchain,
        graph: &ProcessedGraph<Scene, PerFrame, Resources>,
    ) -> Result<(Vec<PassResources>, Vec<usize>), Box<dyn std::error::Error>> {
        //let barriers = Self::generate_barriers(&flat, graph);

        Ok((
            Self::allocate_pass_resources(device, swapchain, &graph)?,
            Self::flatten(&graph)?,
        ))
    }
    pub fn build(
        self,
        gfx: &crate::Gfx,
    ) -> Result<Graph<Scene, PerFrame, Resources>, Box<dyn std::error::Error>> {
        let now = std::time::Instant::now();

        let device = gfx.device();
        let swapchain = &gfx.swapchain().lock();

        let size = self.size.clone().unwrap_or(swapchain.size());

        let graph = self.generate_processed_graph();
        Self::validate(&graph)?;

        let (pass_resources, flat) = Self::compile(device, swapchain, &graph)?;

        let mut allocation_data = AllocationData::new(gfx.device());

        Self::allocate_images(device, swapchain, &mut allocation_data, size, &graph)?;
        Self::allocate_framebuffers(
            device,
            swapchain,
            &mut allocation_data,
            &pass_resources,
            size,
            &graph,
        )?;
        Self::allocate_barriers(&mut allocation_data, &graph);

        log::info!("Graph building took: {:?}", now.elapsed());

        let descriptor_pool = DescriptorPool::new(gfx.device());
        let pipeline_lookup = PipelineLookup::new(device, 100)?;
        Ok(Graph {
            size,
            allocation_data,
            pass_resources,
            graph,
            flat,
            descriptor_pool,
            pipeline_lookup,
        })
        //Graph::compile(self.initial_width, self.initial_height, self.passes)
    }
}
