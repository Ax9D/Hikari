pub mod reflect;
use arrayvec::ArrayVec;
use ash::{prelude::VkResult, vk};
pub use reflect::ReflectionData;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShaderDataType {
    Float,
    Vec2f,
    Vec3f,
    Vec4f,
}
impl ShaderDataType {
    pub const fn size(self) -> usize {
        match self {
            ShaderDataType::Float => 4 * 1,
            ShaderDataType::Vec2f => 4 * 2,
            ShaderDataType::Vec3f => 4 * 3,
            ShaderDataType::Vec4f => 4 * 4,
        }
    }
    pub const fn shape(self) -> u32 {
        match self {
            ShaderDataType::Float => 1,
            ShaderDataType::Vec2f => 2,
            ShaderDataType::Vec3f => 3,
            ShaderDataType::Vec4f => 4,
        }
    }

    pub const fn vk_format(self) -> vk::Format {
        match self {
            ShaderDataType::Float => vk::Format::R32_SFLOAT,
            ShaderDataType::Vec2f => vk::Format::R32G32_SFLOAT,
            ShaderDataType::Vec3f => vk::Format::R32G32B32_SFLOAT,
            ShaderDataType::Vec4f => vk::Format::R32G32B32A32_SFLOAT,
        }
    }
}

use thiserror::Error;

use crate::descriptor::DescriptorSetLayout;
use crate::descriptor::{DescriptorSetLayoutBuilder, MAX_DESCRIPTOR_SETS};

#[derive(Error, Debug)]
pub enum ShaderCreateError {
    #[error("Failed to compile shader, {0}\n Error : {1}")]
    CompilationError(String, String),
    #[error("Failed to validate shader, {0}\n Error: {1}")]
    ValidationError(String, String),
    #[error("Failed to link shader, {0}\n Error : {1}")]
    LinkingError(String, String),
}
#[derive(Clone)]
pub struct ShaderCode<'a> {
    pub entry_point: &'a str,
    pub data: ShaderData,
}

unsafe impl Send for CompiledShaderModule {}
unsafe impl Sync for CompiledShaderModule {}
pub(crate) struct CompiledShaderModule {
    pub debug_name: String,
    pub stage: vk::ShaderStageFlags,
    pub spirv: Vec<u32>,
    pub module: vk::ShaderModule,
    pub reflection_data: reflect::ReflectionData,
}
impl std::hash::Hash for CompiledShaderModule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.spirv.hash(state);
    }
}
impl CompiledShaderModule {
    pub fn create_info(&self) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo::builder()
            .name(const_cstr!("main").as_cstr())
            .stage(self.stage)
            .module(self.module)
            .build()
    }
    pub unsafe fn delete(&self, device: &crate::Device) {
        log::debug!("Deleting shader module");
        device.raw().destroy_shader_module(self.module, None);
    }
}

#[derive(Clone)]
pub enum ShaderData {
    Spirv(Vec<u8>),
    Glsl(String),
}
pub struct Shader {
    device: Arc<crate::Device>,
    name: String,
    modules: Vec<CompiledShaderModule>,
    pipeline_layout: PipelineLayout,

    pub(crate) hash: u64,
}
impl Shader {
    pub fn builder(name: &str) -> ShaderProgramBuilder {
        ShaderProgramBuilder::new(name.to_owned())
    }
    pub(crate) fn vk_stages(&self) -> ArrayVec<vk::PipelineShaderStageCreateInfo, 6> {
        let mut stages = ArrayVec::new();
        for module in &self.modules {
            stages.push(module.create_info());
        }
        stages
    }
    pub(crate) fn pipeline_layout(&self) -> &PipelineLayout {
        &self.pipeline_layout
    }
}
impl std::hash::Hash for Shader {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        //self.name.hash(state);
        // self.vertex.hash(state);
        // self.fragment.hash(state);

        self.hash.hash(state);
    }
}
impl PartialEq for Shader {
    fn eq(&self, other: &Self) -> bool {
        self.modules.iter()
        .zip(other.modules.iter())
        .all(|(current, other)| current.spirv == other.spirv)
    }
}
impl Eq for Shader {}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            for module in &self.modules {
                module.delete(&self.device);
            }

            log::debug!("Dropped shader program");
        }
    }
}
pub(crate) struct PipelineLayout {
    device: Arc<crate::Device>,

    set_layouts: [DescriptorSetLayout; MAX_DESCRIPTOR_SETS],

    set_mask: u32,

    push_constant_stage_flags: vk::ShaderStageFlags,

    vk_pipeline_layout: vk::PipelineLayout,
}

impl PartialEq for PipelineLayout {
    fn eq(&self, other: &Self) -> bool {
        self.set_layouts == other.set_layouts
            && self.set_mask == other.set_mask
            && self.push_constant_stage_flags == other.push_constant_stage_flags
    }
}
impl Eq for PipelineLayout {}

impl PipelineLayout {
    pub fn new(
        device: &Arc<crate::Device>,
        stages: &[CompiledShaderModule],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let set_layouts = Self::generate_descriptor_set_layouts(device, stages)?;
        let push_constant_ranges = Self::generate_push_constant_ranges(stages);

        let push_constant_stage_flags = push_constant_ranges
            .iter()
            .fold(vk::ShaderStageFlags::empty(), |flags, range| {
                flags | range.stage_flags
            });

        let mut vk_set_layouts = [vk::DescriptorSetLayout::null(); MAX_DESCRIPTOR_SETS];

        for (set, set_layout) in set_layouts.iter().enumerate() {
            vk_set_layouts[set] = set_layouts[set].raw();
        }

        assert!(vk_set_layouts
            .iter()
            .all(|&layout| layout != vk::DescriptorSetLayout::null()));

        let vk_pipeline_layout =
            Self::create_pipeline_layout(device, &vk_set_layouts, &push_constant_ranges)?;

        let set_mask = set_layouts
            .iter()
            .enumerate()
            .fold(0, |mask, (set, layout)| {
                if layout.all_mask() != 0 {
                    mask | (1 << set)
                } else {
                    mask
                }
            });

        log::info!("Set mask {:b}", set_mask);

        Ok(Self {
            device: device.clone(),
            set_layouts,
            push_constant_stage_flags,

            vk_pipeline_layout,
            set_mask,
        })
    }
    fn create_pipeline_layout(
        device: &Arc<crate::Device>,
        vk_set_layouts: &[vk::DescriptorSetLayout],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> VkResult<vk::PipelineLayout> {
        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(vk_set_layouts)
            .push_constant_ranges(push_constant_ranges);

        unsafe { device.raw().create_pipeline_layout(&create_info, None) }
    }
    fn generate_descriptor_set_layouts(
        device: &Arc<crate::Device>,
        stages: &[CompiledShaderModule],
    ) -> Result<[DescriptorSetLayout; MAX_DESCRIPTOR_SETS], Box<dyn std::error::Error>> {
        let mut layout_builders: [DescriptorSetLayoutBuilder; MAX_DESCRIPTOR_SETS] =
            [DescriptorSetLayout::builder(); MAX_DESCRIPTOR_SETS];
        for stage in stages {
            for set in &stage
                .reflection_data
                .raw_data()
                .enumerate_descriptor_sets(None)
                .unwrap()
            {
                let layout_builder = &mut layout_builders[set.set as usize];
                for binding in &set.bindings {
                    let descriptor_type =
                        reflect::spirv_desc_type_to_vk_desc_type(binding.descriptor_type);
                    println!("{} {}", binding.name, binding.count);
                    //println!("{:?}", descriptor_type);
                    layout_builder.with_binding(
                        binding.binding,
                        descriptor_type,
                        binding.count,
                        stage.stage,
                    );

                    if let Some((existing_dt, existing_count, _)) =
                        layout_builder.binding(binding.binding)
                    {
                        if existing_dt != descriptor_type || existing_count != binding.count {
                            return Err(format!(
                                "set {} binding {} is different in different stages of shader: {}",
                                set.set, binding.binding, stage.debug_name
                            )
                            .into());
                        }
                    }
                }
            }
        }
        // for (set, layout_builder) in layout_builders.iter().enumerate() {
        //     for binding in 0..MAX_BINDINGS_PER_SET {
        //         print!("{:?} ", layout_builder.binding(binding as u32));
        //     }
        //     println!();
        // }

        let mut layouts = [DescriptorSetLayout::builder().build(device)?; MAX_DESCRIPTOR_SETS];

        layout_builders
            .iter()
            .enumerate()
            .for_each(|(ix, builder)| layouts[ix] = builder.build(device).unwrap());

        Ok(layouts)
    }
    fn generate_push_constant_ranges(
        stages: &[CompiledShaderModule],
    ) -> Vec<vk::PushConstantRange> {
        struct PushConstantRange {
            size: u32,
            offset: u32,
            stage_flags: vk::ShaderStageFlags,
        }

        let mut push_constant_ranges: Vec<PushConstantRange> = Vec::new();
        for stage in stages {
            for block in stage
                .reflection_data
                .raw_data()
                .enumerate_push_constant_blocks(None)
                .unwrap()
            {
                if let Some(existing_range) = push_constant_ranges
                    .iter_mut()
                    .find(|range| range.offset == block.offset && range.size == block.size)
                {
                    existing_range.stage_flags |= stage.stage;
                } else {
                    push_constant_ranges.push(PushConstantRange {
                        size: block.size,
                        offset: block.offset,
                        stage_flags: stage.stage,
                    });
                }
            }
        }

        push_constant_ranges
            .iter()
            .map(|range| {
                vk::PushConstantRange::builder()
                    .size(range.size)
                    .offset(range.offset)
                    .stage_flags(range.stage_flags)
                    .build()
            })
            .collect()
    }
    pub fn raw(&self) -> vk::PipelineLayout {
        self.vk_pipeline_layout
    }

    /// Get a reference to the pipeline layout's set layouts.
    pub fn set_layouts(&self) -> &[DescriptorSetLayout; MAX_DESCRIPTOR_SETS] {
        &self.set_layouts
    }

    pub fn set_mask(&self) -> u32 {
        self.set_mask
    }
    pub fn push_constant_stage_flags(&self) -> vk::ShaderStageFlags {
        self.push_constant_stage_flags
    }
}
impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self.device
                .raw()
                .destroy_pipeline_layout(self.vk_pipeline_layout, None);
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Geometry,
    TessControl,
    TessEvaluation,
    Compute
}
impl std::fmt::Display for ShaderStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderStage::Vertex => write!(f, "Vertex"),
            ShaderStage::Fragment => write!(f, "Fragment"),
            ShaderStage::Geometry => write!(f, "Geometry"),
            ShaderStage::TessControl => write!(f, "TessControl"),
            ShaderStage::TessEvaluation => write!(f, "TessEvaluation"),
            ShaderStage::Compute => write!(f, "Compute"),
        }
    }
}
impl ShaderStage {
    pub(crate) fn shaderc_kind(self) -> shaderc::ShaderKind {
        match self {
            ShaderStage::Vertex => shaderc::ShaderKind::Vertex,
            ShaderStage::Fragment => shaderc::ShaderKind::Fragment,
            ShaderStage::Compute => shaderc::ShaderKind::Compute,
            ShaderStage::Geometry => shaderc::ShaderKind::Geometry,
            ShaderStage::TessControl => shaderc::ShaderKind::TessControl,
            ShaderStage::TessEvaluation => shaderc::ShaderKind::TessEvaluation,
        }
    }
    pub(crate) fn vulkan_stage(self) -> vk::ShaderStageFlags {
        match self {
            ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
            ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
            ShaderStage::Compute => vk::ShaderStageFlags::COMPUTE,
            ShaderStage::Geometry => vk::ShaderStageFlags::GEOMETRY,
            ShaderStage::TessControl => vk::ShaderStageFlags::TESSELLATION_CONTROL,
            ShaderStage::TessEvaluation => vk::ShaderStageFlags::TESSELLATION_EVALUATION,
        }
    }
}
pub struct ShaderProgramBuilder<'entry, 'options> {
    name: String,
    stages: HashMap<ShaderStage, ShaderCode<'entry>>,
    options: Option<shaderc::CompileOptions<'options>>
}

// fn shaderc_to_vulkan_stage(kind: shaderc::ShaderKind) -> vk::ShaderStageFlags {
//     match kind {
//         shaderc::ShaderKind::Vertex => vk::ShaderStageFlags::VERTEX,
//         shaderc::ShaderKind::Fragment => vk::ShaderStageFlags::FRAGMENT,
//         shaderc::ShaderKind::Compute => vk::ShaderStageFlags::COMPUTE,
//         shaderc::ShaderKind::Geometry => vk::ShaderStageFlags::GEOMETRY,
//         shaderc::ShaderKind::TessControl => vk::ShaderStageFlags::TESSELLATION_CONTROL,
//         shaderc::ShaderKind::TessEvaluation => vk::ShaderStageFlags::TESSELLATION_EVALUATION,
//         shaderc::ShaderKind::DefaultVertex => vk::ShaderStageFlags::VERTEX,
//         shaderc::ShaderKind::DefaultFragment => vk::ShaderStageFlags::FRAGMENT,
//         shaderc::ShaderKind::DefaultCompute => vk::ShaderStageFlags::COMPUTE,
//         shaderc::ShaderKind::DefaultGeometry => vk::ShaderStageFlags::GEOMETRY,
//         shaderc::ShaderKind::DefaultTessControl => vk::ShaderStageFlags::TESSELLATION_CONTROL,
//         shaderc::ShaderKind::DefaultTessEvaluation => vk::ShaderStageFlags::TESSELLATION_EVALUATION,

//         //Raytracing **unsupported**
//         // shaderc::ShaderKind::RayGeneration => vk::ShaderStageFlags::RAYGEN_KHR,
//         // shaderc::ShaderKind::AnyHit => vk::ShaderStageFlags::ANY_HIT_KHR,
//         // shaderc::ShaderKind::ClosestHit => vk::ShaderStageFlags::CLOSEST_HIT_KHR,
//         // shaderc::ShaderKind::Miss =>vk::ShaderStageFlags::MISS_KHR,
//         // shaderc::ShaderKind::Intersection => vk::ShaderStageFlags::INTERSECTION_KHR,
//         // shaderc::ShaderKind::Callable => vk::ShaderStageFlags::CALLABLE_KHR,
//         // shaderc::ShaderKind::DefaultRayGeneration => vk::ShaderStageFlags::RAYGEN_KHR,
//         // shaderc::ShaderKind::DefaultAnyHit => vk::ShaderStageFlags::ANY_HIT_KHR,
//         // shaderc::ShaderKind::DefaultClosestHit => vk::ShaderStageFlags::CLOSEST_HIT_KHR,
//         // shaderc::ShaderKind::DefaultMiss => vk::ShaderStageFlags::MISS_KHR,
//         // shaderc::ShaderKind::DefaultIntersection => vk::ShaderStageFlags::INTERSECTION_KHR,
//         // shaderc::ShaderKind::DefaultCallable => vk::ShaderStageFlags::CALLABLE_KHR,

//         //Mesh shading **unsupported**
//         // shaderc::ShaderKind::Task => vk::ShaderStageFlags::TASK_NV,
//         // shaderc::ShaderKind::Mesh => vk::ShaderStageFlags::MESH_NV,
//         // shaderc::ShaderKind::DefaultTask => vk::ShaderStageFlags::TASK_NV,
//         // shaderc::ShaderKind::DefaultMesh => vk::ShaderStageFlags::MESH_NV,
//         _ => panic!("unsupported shader kind"),
//     }
// }

impl<'entry, 'options> ShaderProgramBuilder<'entry, 'options> {
    fn new(name: String) -> Self {
        Self { name, stages: HashMap::new(), options: None }
    } 
    #[deprecated(note = "Use with_stage(...) as it is more general")]
    pub fn vertex_and_fragment(name: &str, vertex: &ShaderCode<'entry>, fragment: &ShaderCode<'entry>) -> Self {
        Self::new(name.to_owned())
        .with_stage(ShaderStage::Vertex, vertex.clone())
        .with_stage(ShaderStage::Fragment, fragment.clone())
    }
    pub fn with_options(mut self, compile_options: shaderc::CompileOptions<'options>) -> Self {
        self.options = Some(compile_options);

        self
    } 
    pub fn with_stage(mut self, stage: ShaderStage, code: ShaderCode<'entry>) -> Self {
        self.stages.insert(stage, code);

        self
    }
    fn compile_shader(
        compiler: &mut shaderc::Compiler,
        glsl: &str,
        entry_point: &str,
        stage: ShaderStage,
        debug_name: &str,
        options: Option<&shaderc::CompileOptions>
    ) -> Result<Vec<u32>, ShaderCreateError> {
        #[allow(unused_mut)]
        //options.set_optimization_level(shaderc::OptimizationLevel::Zero);
        let artifact = compiler
            .compile_into_spirv(glsl, stage.shaderc_kind(), debug_name, entry_point, options)
            .map_err(|err| {
                ShaderCreateError::CompilationError(debug_name.to_string(), err.to_string())
            })?;

        log::debug!("Compiled shader {}", debug_name);

        if artifact.get_num_warnings() > 0 {
            log::warn!(
                "[Shader Compiler]({}) {}",
                debug_name,
                artifact.get_warning_messages()
            );
        }

        Ok(artifact.as_binary().to_vec())
    }
    fn create_vk_module(device: &ash::Device, code: &[u32]) -> VkResult<vk::ShaderModule> {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(code).build();

        log::debug!("Created shader module");
        unsafe { device.create_shader_module(&create_info, None) }
    }

    fn create_shader_module(
        device: &crate::Device,
        shader: &ShaderCode<'entry>,
        debug_name: String,
        stage: ShaderStage,
        options: Option<&shaderc::CompileOptions>
    ) -> Result<CompiledShaderModule, ShaderCreateError> {
        let data;
        let spirv = match &shader.data {
            ShaderData::Spirv(data) => unsafe {
                let ptr_u32 = data.as_ptr() as *const u32;
                let len = data.len() / (std::mem::size_of::<u32>() / std::mem::size_of::<u8>());

                std::slice::from_raw_parts(ptr_u32, len)
            },
            ShaderData::Glsl(glsl) => {
                data = Self::compile_shader(
                    &mut device.shader_compiler(),
                    glsl,
                    shader.entry_point,
                    stage,
                    &debug_name,
                    options
                )?;
                &data
            }
        };

        let reflection_data = super::ReflectionData::new(spirv)
            .map_err(|err| ShaderCreateError::ValidationError(debug_name.clone(), err))?;

        let module = Self::create_vk_module(device.raw(), spirv).map_err(|error| {
            ShaderCreateError::CompilationError(debug_name.clone(), error.to_string())
        })?;

        let stage = stage.vulkan_stage();

        Ok(CompiledShaderModule {
            debug_name,
            stage,
            module,
            spirv: spirv.to_vec(),
            reflection_data,
        })
    }
    pub fn build(mut self, device: &Arc<crate::Device>) -> Result<Arc<Shader>, ShaderCreateError> {
        log::debug!("Compiling vertex shader");

        let mut modules = vec![];

        for (stage, code) in self.stages.drain() {
            let module = Self::create_shader_module(device, &code, format!("[{}] {}", stage.to_string(), self.name), stage, self.options.as_ref())?;
            modules.push(module);
        }

        let pipeline_layout = PipelineLayout::new(device, &modules)
            .map_err(|err| ShaderCreateError::LinkingError(self.name.clone(), err.to_string()))?;

        let mut hasher = crate::util::hasher();

        let hash = {
            for module in &modules {
                module.hash(&mut hasher);
            }

            hasher.finish()
        };

        Ok(Arc::new(Shader {
            name: self.name,
            device: device.clone(),
            modules,
            pipeline_layout,

            hash,
        }))
    }
}