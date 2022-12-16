use vk_sync_fork::AccessType;

use crate::{graph::{GpuHandle, command::{render::PassRecordInfo, compute::ComputepassCommands}}, texture::SampledImage, Args, ByRef, Buffer, GraphResources};

use super::{Input, Output};

pub struct ComputePass<T: Args> {
    name: String,
    id: u64,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    pub(crate) record_fn:
        Option<Box<dyn FnMut(&mut ComputepassCommands, &GraphResources, &PassRecordInfo, <T::Ref as ByRef>::Item) + Send + Sync>>,
}

impl<T: Args> ComputePass<T> {
    /// Creates a new Renderpass
    /// A name should be provided for debug usage
    /// `record_fn` is a closure which is used to record rendering commands when the renderpass is executed
    pub fn new(
        name: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            id: crate::util::quick_hash(name),
            inputs: Vec::new(),
            outputs: Vec::new(),
            record_fn: None,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }
    fn read_image_check(&mut self, image: &GpuHandle<SampledImage>, access_type: AccessType) {
        if self.inputs.iter().any(|input| match input {
            Input::ReadImage(existing_image, _) => {
                existing_image == image
            }
            _=> false
        }) {
            panic!("Image handle {:?} already registered for read", image);
        }

        match access_type {
            AccessType::ComputeShaderReadSampledImageOrUniformTexelBuffer |
            AccessType::Nothing |
            AccessType::VertexShaderReadSampledImageOrUniformTexelBuffer |
            AccessType::VertexShaderReadOther |
            AccessType::TessellationControlShaderReadSampledImageOrUniformTexelBuffer |
            AccessType::TessellationControlShaderReadOther |
            AccessType::TessellationEvaluationShaderReadSampledImageOrUniformTexelBuffer |
            AccessType::TessellationEvaluationShaderReadOther |
            AccessType::GeometryShaderReadSampledImageOrUniformTexelBuffer |
            AccessType::GeometryShaderReadOther |
            AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer |
            AccessType::FragmentShaderReadColorInputAttachment |
            AccessType::FragmentShaderReadDepthStencilInputAttachment |
            AccessType::FragmentShaderReadOther |
            AccessType::ColorAttachmentRead |
            AccessType::DepthStencilAttachmentRead |
            AccessType::ComputeShaderReadOther |
            AccessType::AnyShaderReadSampledImageOrUniformTexelBuffer |
            AccessType::AnyShaderReadOther => {},
            _ => panic!(
                "Invalid access type {:?} for image read in computepass",
                access_type
            ),
        }
    }
    fn write_image_check(&mut self, image: &GpuHandle<SampledImage>, access_type: AccessType) {
        if self.outputs.iter().any(|input| match input {
            Output::WriteImage(existing_image, _) => {
                existing_image == image
            }
            _=> false
        }) {
            panic!("Image handle {:?} already registered for write", image);
        }

        match access_type {
            AccessType::VertexShaderWrite |
            AccessType::TessellationControlShaderWrite |
            AccessType::TessellationEvaluationShaderWrite |
            AccessType::GeometryShaderWrite |
            AccessType::FragmentShaderWrite |
            AccessType::ComputeShaderWrite |
            AccessType::AnyShaderWrite |
            AccessType::TransferWrite |
            AccessType::HostWrite |
            AccessType::General => {}

            _ => panic!(
                "Invalid access type {:?} for image write in computepass",
                access_type
            ),
        }
    }
    pub fn cmd(mut self, record_fn: impl FnMut(&mut ComputepassCommands, &GraphResources, &PassRecordInfo, <T::Ref as ByRef>::Item) + Send + Sync + 'static) -> Self {
        self.record_fn = Some(Box::new(record_fn));

        self
    }
    pub fn read_image(mut self, image: &GpuHandle<SampledImage>, access_type: AccessType) -> Self {
        self.read_image_check(image, access_type);

        self.inputs
            .push(Input::ReadImage(image.clone(), access_type));
        self
    }
    pub fn write_image(mut self, image: &GpuHandle<SampledImage>, access_type: AccessType) -> Self {
        self.write_image_check(image, access_type);

        self.outputs.push(Output::WriteImage(image.clone(), access_type));
        self
    }
    pub fn read_buffer<B: Buffer + 'static>(mut self, buffer: &GpuHandle<B>, access_type: AccessType) -> Self {
        self.inputs.push(Input::ReadStorageBuffer(buffer.clone().into(), access_type));

        self
    }
    pub fn write_buffer<B: Buffer + 'static>(mut self, buffer: &GpuHandle<B>, access_type: AccessType) -> Self {
        self.outputs.push(Output::WriteStorageBuffer(buffer.clone().into(), access_type));

        self
    }
}
