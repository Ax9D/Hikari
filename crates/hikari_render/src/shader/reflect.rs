use ash::vk;

// #[derive(Debug, Clone)]
// pub struct Attribute {
//     pub name: String,
//     pub location: u32,
// }
// #[derive(Debug, Clone)]
// pub struct UniformBuffer {
//     pub name: String,
//     pub set: u32,
//     pub binding: u32,
// }

// #[derive(Debug, Clone)]
// pub struct PushConstantRange {
//     pub size: u32,
//     pub offset: u32,
// }

// #[derive(Debug, Clone)]
// pub struct CombinedImageSampler {
//     pub name: String,
//     pub set: u32,
//     pub binding: u32,
//     pub count: usize,
// }
#[derive(Clone)]
pub struct ReflectionData {
    raw: spirv_reflect::ShaderModule,
}

impl ReflectionData {
    // pub fn inputs(&self) -> &HashMap<String, Attribute> {
    //     &self.inputs
    // }
    // pub fn outputs(&self) -> &HashMap<String, Attribute> {
    //     &self.outputs
    // }
    // pub fn uniform_buffers(&self) -> &HashMap<String, UniformBuffer> {
    //     &self.uniform_buffers
    // }
    // pub fn push_constant(&self) -> &Option<PushConstantRange> {
    //     &self.push_constant
    // }
    // pub fn combined_image_samplers(&self) -> &HashMap<String, CombinedImageSampler> {
    //     &self.combined_image_samplers
    // }
    pub fn raw_data(&self) -> &spirv_reflect::ShaderModule {
        &self.raw
    }
}
pub fn spirv_desc_type_to_vk_desc_type(
    spv: spirv_reflect::types::descriptor::ReflectDescriptorType,
) -> vk::DescriptorType {
    match spv {
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
        spirv_reflect::types::ReflectDescriptorType::AccelerationStructureNV => {
            vk::DescriptorType::ACCELERATION_STRUCTURE_NV
        }
    }
}

impl ReflectionData {
    pub fn new(spirv: &[u32]) -> Result<ReflectionData, String> {
        // let mut inputs = HashMap::new();
        // let mut outputs = HashMap::new();

        let shader_module = spirv_reflect::ShaderModule::load_u32_data(spirv)?;
        Ok(Self { raw: shader_module })
        // let entry_point = None;
        // for input in shader_module.enumerate_input_variables(entry_point)? {
        //     inputs.insert(
        //         input.name.clone(),
        //         Attribute {
        //             name: input.name,
        //             location: input.location,
        //         },
        //     );
        // }
        // for output in shader_module.enumerate_output_variables(entry_point)? {
        //     outputs.insert(
        //         output.name.clone(),
        //         Attribute {
        //             name: output.name,
        //             location: output.location,
        //         },
        //     );
        // }
        // let push_constants = shader_module.enumerate_push_constant_blocks(entry_point)?;
        // let push_constants = push_constants.first();

        // let push_constant = push_constants.map(|push_constant| PushConstantRange {
        //     size: push_constant.size,
        //     offset: push_constant.offset,
        // });

        // let mut uniform_buffers = HashMap::new();
        // let mut combined_image_samplers = HashMap::new();

        // for binding in shader_module.enumerate_descriptor_bindings(entry_point)? {
        //     //log::debug!("{:?}", binding.name);

        //     if binding.set as usize >= crate::descriptor::MAX_DESCRIPTOR_SETS {
        //         return Err(format!(
        //             "annot have more than {} descriptor sets",
        //             crate::descriptor::MAX_DESCRIPTOR_SETS
        //         ));
        //     }

        //     if binding.binding as usize >= crate::descriptor::MAX_BINDINGS_PER_SET {
        //         return Err(format!(
        //             "annot have more than {} bindings in a set",
        //             crate::descriptor::MAX_BINDINGS_PER_SET
        //         ));
        //     }

        //     //Descriptor set name can be a empty string for some reason????
        //     if !binding.name.is_empty() {
        //         match binding.descriptor_type {
        //         spirv_reflect::types::ReflectDescriptorType::CombinedImageSampler => {
        //             combined_image_samplers.insert(binding.name.clone(),
        //             CombinedImageSampler {
        //                 name: binding.name,
        //                 set: binding.set,
        //                 binding: binding.binding,
        //                 count: binding.array.dims.iter().product::<u32>() as usize
        //             }
        //         );
        //         },
        //         spirv_reflect::types::ReflectDescriptorType::UniformBuffer => {
        //             uniform_buffers.insert(binding.name.clone(),
        //             UniformBuffer {
        //                 name: binding.name,
        //                 binding: binding.binding,
        //                 set: binding.set,
        //             });
        //         },
        //         spirv_reflect::types::ReflectDescriptorType::StorageBuffer => {

        //         }
        //         _=> return Err(format!("Unsupported descriptor type on {}. Only uniform buffers, push constants and combined image samplers are supported currently", binding.name))
        //     }
        //     }
        // }

        // // let uniforms_in_blocks = Self::reflect_uniform_blocks(program);
        // // let uniforms = Self::reflect_uniforms(program, &uniforms_in_blocks);

        // inputs.iter().for_each(|(_, x)| {
        //     //log::debug!("{:?}", x);
        // });
        // outputs.iter().for_each(|(_, x)| {
        //     //log::debug!("{:?}", x);
        // });
        // uniform_buffers.iter().for_each(|(_, x)| {
        //     //log::debug!("{:?}", x);
        // });

        // match push_constant {
        //     Some(ref pc) => {
        //         //log::debug!("{:?}", pc)
        //     }
        //     None => {
        //         //log::debug!("No push constants")
        //     }
        // }

        // combined_image_samplers.iter().for_each(|(_, x)| {
        //     //log::debug!("{:?}", x);
        // });

        // println!("Done");
        // Ok(Self {
        //     inputs,
        //     outputs,
        //     uniform_buffers,
        //     push_constant,
        //     combined_image_samplers,
        //     raw: shader_module,
        // })
    }
}
