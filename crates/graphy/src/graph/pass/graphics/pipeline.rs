use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use ash::vk;

use crate::ShaderDataType;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PrimitiveTopology {
    Point = 0,
    Triangles = 3,
    Lines = 1,
}
impl PrimitiveTopology {
    pub fn into_vk(&self) -> vk::PrimitiveTopology {
        vk::PrimitiveTopology::from_raw(*self as i32)
    }
}
impl Default for PrimitiveTopology {
    fn default() -> Self {
        Self::Triangles
    }
}
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PolygonMode {
    Fill = 0,
    Line,
    Point,
}
impl PolygonMode {
    pub fn into_vk(&self) -> vk::PolygonMode {
        vk::PolygonMode::from_raw(*self as i32)
    }
}

impl Default for PolygonMode {
    fn default() -> Self {
        Self::Fill
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompareOp {
    Never = 0,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}
impl CompareOp {
    pub fn into_vk(&self) -> vk::CompareOp {
        vk::CompareOp::from_raw(*self as i32)
    }
}

impl Default for CompareOp {
    fn default() -> Self {
        Self::Never
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StencilOp {
    Keep = 0,
    Zero,
    Replace,
    IncrementAndClamp,
    DecrementAndClamp,
    Invert,
    IncrementAndWrap,
    DecrementAndWrap,
}
impl Default for StencilOp {
    fn default() -> Self {
        Self::Keep
    }
}
impl StencilOp {
    pub fn into_vk(&self) -> vk::StencilOp {
        vk::StencilOp::from_raw(*self as i32)
    }
}
fn bad_float_equal_rep(x: f32) -> u32 {
    let rounded = (x * 100.0).round() / 100.0;
    rounded.to_bits()
}
fn bad_float_hash(x: f32, hasher: &mut impl std::hash::Hasher) {
    bad_float_equal_rep(x).hash(hasher);
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DepthStencilState {
    pub depth_test_enabled: bool,
    pub depth_write_enabled: bool,
    pub depth_compare_op: CompareOp,

    pub depth_bound_test_enabled: bool,

    pub stencil_test_enabled: bool,
    pub stencil_test_compare_op: CompareOp,
    pub stencil_test_fail_op: StencilOp,
    pub stencil_test_depth_fail_op: StencilOp,
    pub stencil_test_pass_op: StencilOp,
    pub stencil_test_compare_mask: u32,
    pub stencil_test_write_mark: u32,
    pub stencil_test_reference: u32,

    pub min_depth_bounds: f32,
    pub max_depth_bounds: f32,
}
impl std::hash::Hash for DepthStencilState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.depth_test_enabled.hash(state);
        self.depth_write_enabled.hash(state);
        self.depth_compare_op.hash(state);
        self.depth_bound_test_enabled.hash(state);
        self.stencil_test_enabled.hash(state);
        self.stencil_test_compare_op.hash(state);
        self.stencil_test_fail_op.hash(state);
        self.stencil_test_depth_fail_op.hash(state);
        self.stencil_test_pass_op.hash(state);
        self.stencil_test_compare_mask.hash(state);
        self.stencil_test_write_mark.hash(state);
        self.stencil_test_reference.hash(state);

        bad_float_hash(self.min_depth_bounds, state);
        bad_float_hash(self.max_depth_bounds, state);
    }
}

impl Eq for DepthStencilState {}

impl DepthStencilState {
    pub fn into_vk(&self) -> vk::PipelineDepthStencilStateCreateInfo {
        let stencil_op = *vk::StencilOpState::builder()
            .compare_op(self.stencil_test_compare_op.into_vk())
            .fail_op(self.stencil_test_fail_op.into_vk())
            .depth_fail_op(self.stencil_test_depth_fail_op.into_vk())
            .pass_op(self.stencil_test_pass_op.into_vk())
            .compare_mask(self.stencil_test_compare_mask)
            .write_mask(self.stencil_test_write_mark)
            .reference(self.stencil_test_reference);

        *vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(self.depth_test_enabled)
            .depth_write_enable(self.depth_write_enabled)
            .depth_compare_op(self.depth_compare_op.into_vk())
            .depth_bounds_test_enable(self.depth_bound_test_enabled)
            .stencil_test_enable(self.stencil_test_enabled)
            .front(stencil_op)
            .back(stencil_op)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RasterizerState {
    pub polygon_mode: PolygonMode,

    pub depth_clamp_enable: bool,
    pub rasterizer_discard_enable: bool,

    pub depth_bias_enable: bool,
    pub depth_bias_constant_factor: f32,
    pub depth_bias_clamp: f32,
    pub depth_bias_slope_factor: f32,
}
impl RasterizerState {
    pub fn into_vk(&self) -> vk::PipelineRasterizationStateCreateInfo {
        *vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(self.depth_clamp_enable)
            .rasterizer_discard_enable(self.rasterizer_discard_enable)
            .polygon_mode(self.polygon_mode.into_vk())
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(self.depth_bias_enable)
            .depth_bias_constant_factor(self.depth_bias_constant_factor)
            .depth_bias_clamp(self.depth_bias_clamp)
            .depth_bias_slope_factor(self.depth_bias_slope_factor)
    }
}

impl Eq for RasterizerState {}

impl Hash for RasterizerState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.polygon_mode.hash(state);
        self.depth_clamp_enable.hash(state);
        self.rasterizer_discard_enable.hash(state);
        self.depth_bias_enable.hash(state);

        bad_float_hash(self.depth_bias_constant_factor, state);
        bad_float_hash(self.depth_bias_clamp, state);
        bad_float_hash(self.depth_bias_slope_factor, state);
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BlendFactor {
    Zero = 0,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColor,
}
impl Default for BlendFactor {
    fn default() -> Self {
        Self::Zero
    }
}
impl BlendFactor {
    pub fn into_vk(&self) -> vk::BlendFactor {
        vk::BlendFactor::from_raw(*self as i32)
    }
}
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BlendOp {
    Add = 0,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}
impl Default for BlendOp {
    fn default() -> Self {
        Self::Add
    }
}
impl BlendOp {
    pub fn into_vk(&self) -> vk::BlendOp {
        vk::BlendOp::from_raw(*self as i32)
    }
}

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct BlendState {
    pub enabled: bool,
    pub src_color_blend_factor: BlendFactor,
    pub dst_color_blend_factor: BlendFactor,
    pub color_blend_op: BlendOp,
    pub src_alpha_blend_factor: BlendFactor,
    pub dst_alpha_blend_factor: BlendFactor,
    pub alpha_blend_op: BlendOp,
}

impl BlendState {
    pub fn into_vk(&self) -> vk::PipelineColorBlendAttachmentState {
        *vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(self.enabled)
            .src_color_blend_factor(self.src_color_blend_factor.into_vk())
            .dst_color_blend_factor(self.dst_color_blend_factor.into_vk())
            .src_alpha_blend_factor(self.src_alpha_blend_factor.into_vk())
            .dst_alpha_blend_factor(self.dst_alpha_blend_factor.into_vk())
            .alpha_blend_op(self.alpha_blend_op.into_vk())
    }
}

use crate::util::ArrayVecCopy;
const MAX_VERTEX_BINDINGS: usize = 4;
const MAX_VERTEX_ATTRIBUTES: usize = 8;
#[derive(Debug, Default, Clone, Copy)]
pub struct VertexInputLayout {
    pub binding_descs: ArrayVecCopy<vk::VertexInputBindingDescription, MAX_VERTEX_BINDINGS>,
    pub attribute_descs: ArrayVecCopy<vk::VertexInputAttributeDescription, MAX_VERTEX_ATTRIBUTES>,
}

impl PartialEq for VertexInputLayout {
    fn eq(&self, other: &Self) -> bool {
        let bindings_same = self
            .binding_descs
            .iter()
            .zip(other.binding_descs.iter())
            .all(|(this, other)| {
                this.binding == other.binding
                    && this.stride == other.stride
                    && this.input_rate == other.input_rate
            });

        let attributes_same = self
            .attribute_descs
            .iter()
            .zip(other.attribute_descs.iter())
            .all(|(this, other)| {
                this.binding == other.binding
                    && this.location == other.location
                    && this.format == other.format
                    && this.offset == other.offset
            });

        bindings_same && attributes_same
    }
}
impl Eq for VertexInputLayout {}

impl Hash for VertexInputLayout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for binding_desc in &self.binding_descs {
            binding_desc.binding.hash(state);
            binding_desc.input_rate.hash(state);
            binding_desc.stride.hash(state);
        }
        for attribute in &self.attribute_descs {
            attribute.binding.hash(state);
            attribute.location.hash(state);
            attribute.format.hash(state);
            attribute.offset.hash(state);
        }
    }
}

impl VertexInputLayout {
    pub fn into_vk(&self) -> vk::PipelineVertexInputStateCreateInfo {
        *vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&self.attribute_descs)
            .vertex_binding_descriptions(&self.binding_descs)
    }
}

impl VertexInputLayout {
    pub fn new() -> VertexInputLayoutBuilder {
        VertexInputLayoutBuilder {
            layouts: Vec::new(),
        }
    }
}
#[derive(Copy, Clone, Debug)]
pub enum StepMode {
    Vertex,
    Instance,
}
impl Default for StepMode {
    fn default() -> Self {
        Self::Vertex
    }
}

impl From<StepMode> for vk::VertexInputRate {
    fn from(step: StepMode) -> Self {
        match step {
            StepMode::Vertex => vk::VertexInputRate::VERTEX,
            StepMode::Instance => vk::VertexInputRate::INSTANCE,
        }
    }
}

pub struct VertexInputLayoutBuilder {
    layouts: Vec<(Vec<ShaderDataType>, StepMode)>,
}
impl VertexInputLayoutBuilder {
    pub fn buffer(mut self, layout: &[ShaderDataType], step_mode: StepMode) -> Self {
        self.layouts.push((layout.to_vec(), step_mode));

        self
    }
    pub fn build(self) -> VertexInputLayout {
        let mut binding_descs = ArrayVecCopy::new();
        let mut attribute_descs = ArrayVecCopy::new();

        let mut location = 0;
        for (binding, (layout, step_mode)) in self.layouts.iter().enumerate() {
            binding_descs.push(
                *vk::VertexInputBindingDescription::builder()
                    .binding(binding as u32)
                    .stride(layout.iter().fold(0, |acc, x| acc + x.size() as u32))
                    .input_rate((*step_mode).into()), //
            );

            let mut offset = 0;
            for field in layout {
                attribute_descs.push(
                    *vk::VertexInputAttributeDescription::builder()
                        .binding(binding as u32)
                        .location(location)
                        .format(field.vk_format())
                        .offset(offset),
                );

                offset += field.size() as u32;
                location += 1;
            }
        }

        VertexInputLayout {
            binding_descs,
            attribute_descs,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PipelineState {
    pub input_layout: VertexInputLayout,
    pub primitive_topology: PrimitiveTopology,
    pub rasterizer_state: RasterizerState,
    pub depth_stencil_state: DepthStencilState,
    pub blend_state: BlendState,
}
impl std::hash::Hash for PipelineState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        hikari_dev::profile_function!();

        self.primitive_topology.hash(state);
        self.rasterizer_state.hash(state);
        self.depth_stencil_state.hash(state);
        self.blend_state.hash(state);
        self.input_layout.hash(state);
    }
}

impl PipelineState {
    pub unsafe fn create_pipeline(
        &self,
        device: &Arc<crate::Device>,
        shader: &crate::Shader,
        renderpass: vk::RenderPass,
        n_color_attachments: usize,
    ) -> vk::Pipeline {
        hikari_dev::profile_function!();
        let now = std::time::Instant::now();

        let pipeline_cache = device.pipeline_cache();

        let input_state = self.input_layout.into_vk();

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(self.primitive_topology.into_vk())
            .primitive_restart_enable(false);

        let rasterizer = self.rasterizer_state.into_vk();
        let blend_attachment = vec![self.blend_state.into_vk(); n_color_attachments];

        let color_blend = *vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&blend_attachment)
            .blend_constants([0.0; 4]);

        let multi_sample = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .flags(vk::PipelineMultisampleStateCreateFlags::empty());

        let viewport = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1)
            .flags(vk::PipelineViewportStateCreateFlags::empty());

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_state)
            .flags(vk::PipelineDynamicStateCreateFlags::empty());

        let stages = shader.vk_stages();

        let depth_stencil = self.depth_stencil_state.into_vk();

        let layout = shader.pipeline_layout().raw();

        let create_info = vk::GraphicsPipelineCreateInfo::builder()
            .input_assembly_state(&input_assembly)
            .stages(&stages)
            .render_pass(renderpass)
            .vertex_input_state(&input_state)
            .viewport_state(&viewport)
            .rasterization_state(&rasterizer)
            .multisample_state(&multi_sample)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic_state)
            .layout(layout);

        log::debug!("Creating pipeline: {:#?}", self);
        let create_infos = [*create_info];
        let pipeline = device
            .raw()
            .create_graphics_pipelines(pipeline_cache, &create_infos, None)
            .unwrap()[0];

        log::debug!("Pipeline creation took: {:?}", now.elapsed());
        pipeline
    }
}
