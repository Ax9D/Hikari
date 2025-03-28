use arrayvec::ArrayVec;
use ash::{prelude::VkResult, vk};
use shaderc::CompileOptions;

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

use crate::descriptor::{DescriptorSetLayout, BINDLESS_SET_ID};
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
    pub reflection_data: spirv_reflect::ShaderModule,
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
    #[inline]
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
        self.modules
            .iter()
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

        //log::info!("Set mask {:b}", set_mask);

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
    
    fn spirv_desc_type_to_vk_desc_type(spirv_type: spirv_reflect::types::ReflectDescriptorType) -> vk::DescriptorType {
        match spirv_type {
            spirv_reflect::types::ReflectDescriptorType::Undefined => todo!(),
            spirv_reflect::types::ReflectDescriptorType::Sampler => vk::DescriptorType::SAMPLER,
            spirv_reflect::types::ReflectDescriptorType::CombinedImageSampler => {
                vk::DescriptorType::COMBINED_IMAGE_SAMPLER
            }
            spirv_reflect::types::ReflectDescriptorType::SampledImage => {
                vk::DescriptorType::SAMPLED_IMAGE
            }
            spirv_reflect::types::ReflectDescriptorType::StorageImage => {
                vk::DescriptorType::STORAGE_IMAGE
            }
            spirv_reflect::types::ReflectDescriptorType::UniformTexelBuffer => {
                vk::DescriptorType::UNIFORM_TEXEL_BUFFER
            }
            spirv_reflect::types::ReflectDescriptorType::StorageTexelBuffer => {
                vk::DescriptorType::STORAGE_TEXEL_BUFFER
            }
            spirv_reflect::types::ReflectDescriptorType::UniformBuffer => {
                vk::DescriptorType::UNIFORM_BUFFER
            }
            spirv_reflect::types::ReflectDescriptorType::StorageBuffer => {
                vk::DescriptorType::STORAGE_BUFFER
            }
            spirv_reflect::types::ReflectDescriptorType::UniformBufferDynamic => {
                vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC
            }
            spirv_reflect::types::ReflectDescriptorType::StorageBufferDynamic => {
                vk::DescriptorType::STORAGE_BUFFER_DYNAMIC
            }
            spirv_reflect::types::ReflectDescriptorType::InputAttachment => {
                vk::DescriptorType::INPUT_ATTACHMENT
            }
            spirv_reflect::types::ReflectDescriptorType::AccelerationStructureKHR => {
                vk::DescriptorType::ACCELERATION_STRUCTURE_KHR
            }
        }
    }
    fn generate_descriptor_set_layouts(
        device: &Arc<crate::Device>,
        stages: &[CompiledShaderModule],
    ) -> anyhow::Result<[DescriptorSetLayout; MAX_DESCRIPTOR_SETS]> {
        let mut layout_builders: [DescriptorSetLayoutBuilder; MAX_DESCRIPTOR_SETS] =
            [DescriptorSetLayout::builder(); MAX_DESCRIPTOR_SETS];
        
        let mut has_bindless = false;
        for stage in stages {
            for set_info in &stage
                .reflection_data
                .enumerate_descriptor_sets(None)
                .unwrap()
            {
                if set_info.set == BINDLESS_SET_ID {
                    has_bindless = true;
                    continue;
                }
                
                let layout_builder = &mut layout_builders[set_info.set as usize];
                for binding_info in &set_info.bindings {
                    let descriptor_type = Self::spirv_desc_type_to_vk_desc_type(binding_info.descriptor_type);
                    //println!("{} {} {:?} {:#?}", &binding_info.name, binding_info.binding, binding_info.count, binding_info.type_description);

                    let count = binding_info.count;
                    let stage_flags = stage.stage;

                    let binding_flags = vk::DescriptorBindingFlags::empty();

                    // if is_bindless {
                    //     binding_flags |= vk::DescriptorBindingFlags::PARTIALLY_BOUND; 
                    //     binding_flags |=vk::DescriptorBindingFlags::UPDATE_AFTER_BIND; 
                    //     binding_flags |= vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING;

                    //     layout_builder.create_flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
                        
                    //     stage_flags = vk::ShaderStageFlags::ALL;
                    //     count = MAX_BINDLESS_COUNT as u32;
                    // }

                    layout_builder.with_binding(
                        binding_info.binding,
                        descriptor_type,
                        count,
                        stage_flags,
                        binding_flags
                    );

                    if let Some((existing_dt, existing_count, _, _)) =
                        layout_builder.binding(binding_info.binding)
                    {
                        if existing_dt != descriptor_type || existing_count != count {
                            return Err(anyhow::anyhow!(
                                "set {} binding {} is different in different stages of shader: {}",
                                set_info.set, binding_info.binding, stage.debug_name
                            ));
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

        if has_bindless {
            layouts[BINDLESS_SET_ID as usize] = *device.bindless_resources().set_layout();
        }

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
    #[inline]
    pub fn set_layouts(&self) -> &[DescriptorSetLayout; MAX_DESCRIPTOR_SETS] {
        &self.set_layouts
    }
    #[inline]
    pub fn set_mask(&self) -> u32 {
        self.set_mask
    }
    #[inline]
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
    Compute,
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
pub struct ShaderProgramBuilder<'entry, 'defines> {
    name: String,
    stages: HashMap<ShaderStage, (ShaderCode<'entry>,  Vec<&'defines str>)>,
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

impl<'entry, 'defines> ShaderProgramBuilder<'entry, 'defines> {
    fn new(name: String) -> Self {
        Self {
            name,
            stages: HashMap::new(),
        }
    }
    #[deprecated(note = "Use with_stage(...) as it is more general")]
    pub fn vertex_and_fragment(
        name: &str,
        vertex: &ShaderCode<'entry>,
        fragment: &ShaderCode<'entry>,
    ) -> Self {
        Self::new(name.to_owned())
            .with_stage(ShaderStage::Vertex, vertex.clone(), &[])
            .with_stage(ShaderStage::Fragment, fragment.clone(), &[])
    }
    pub fn with_stage(
        mut self,
        stage: ShaderStage,
        code: ShaderCode<'entry>,
        defines: &[&'defines str],
    ) -> Self {
        self.stages.insert(stage, (code, defines.to_vec()));

        self
    }
    fn compile_shader(
        compiler: &mut shaderc::Compiler,
        glsl: &str,
        entry_point: &str,
        stage: ShaderStage,
        debug_name: &str,
        options: Option<&shaderc::CompileOptions>,
    ) -> Result<Vec<u32>, ShaderCreateError> {
        #[allow(unused_mut)]
        //options.set_optimization_level(shaderc::OptimizationLevel::Zero);
        let artifact = compiler
            .compile_into_spirv(glsl, stage.shaderc_kind(), debug_name, entry_point, options)
            .map_err(|err| {
                ShaderCreateError::CompilationError(debug_name.to_string(), err.to_string())
            })?;

        //log::debug!("Compiled shader {}", debug_name);

        if artifact.get_num_warnings() > 0 {
            log::warn!(
                "[Shader Compiler]({}) {}",
                debug_name,
                artifact.get_warning_messages()
            );
        }
        let data = artifact.as_binary();

        Ok(data.to_vec())
    }
    fn create_vk_module(device: &ash::Device, code: &[u32]) -> VkResult<vk::ShaderModule> {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(code).build();

        unsafe { device.create_shader_module(&create_info, None) }
    }

    fn create_shader_module(
        device: &crate::Device,
        shader: &ShaderCode<'entry>,
        debug_name: String,
        stage: ShaderStage,
        options: Option<&CompileOptions>,
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
                    options,
                )?;

                &data
            }
        };

        
        let module = Self::create_vk_module(device.raw(), spirv).map_err(|error| {
            ShaderCreateError::CompilationError(debug_name.clone(), error.to_string())
        })?;

        let reflection_data = spirv_reflect::ShaderModule::load_u32_data(spirv)
        .map_err(|err| ShaderCreateError::ValidationError(debug_name.clone(), err.to_string()))?;
        
        let stage = stage.vulkan_stage();

        Ok(CompiledShaderModule {
            debug_name,
            stage,
            module,
            spirv: spirv.to_vec(),
            reflection_data,
        })
    }
    pub fn build(
        mut self,
        device: &Arc<crate::Device>,
        options: Option<CompileOptions>,
    ) -> Result<Arc<Shader>, ShaderCreateError> {
        let mut modules = vec![];

        let compile_options = options.unwrap_or_else(|| CompileOptions::new().unwrap());

        for (stage, (code, defines)) in self.stages.drain() {
            let mut compile_options = compile_options.clone().unwrap();

            for define in defines {
                compile_options.add_macro_definition(define, None);
            }
            let module = Self::create_shader_module(
                device,
                &code,
                format!("[{}] {}", stage.to_string(), self.name),
                stage,
                Some(&compile_options),
            )?;
            modules.push(module);
        }

        let pipeline_layout = PipelineLayout::new(device, &modules)
            .map_err(|err| ShaderCreateError::LinkingError(self.name.clone(), err.to_string()))?;

        let mut hasher = crate::util::hasher();

        log::debug!("Compiled {}", self.name);

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
