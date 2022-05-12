pub mod pipeline;
use vk_sync_fork::AccessType;

use crate::{graph::Handle, texture::SampledImage, Args, ByRef, RenderpassCommands};

use super::{AttachmentConfig, ImageSize, Input, Output};

pub use pipeline::*;

pub struct Renderpass<T: Args> {
    name: String,
    id: u64,
    pub(crate) render_area: ImageSize,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    pub(crate) present_to_swapchain: bool,
    pub(crate) record_fn:
        Box<dyn FnMut(&mut RenderpassCommands, <T::Ref as ByRef>::Item) + Send + Sync>,
}

impl<T: Args> Renderpass<T> {
    /// Creates a new Renderpass
    /// A name should be provided for debug usage
    /// `record_fn` is a closure which is used to record rendering commands when the renderpass is executed
    pub fn new(
        name: &str,
        area: ImageSize,
        record_fn: impl FnMut(&mut RenderpassCommands, <T::Ref as ByRef>::Item) + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            id: crate::util::quick_hash(name),
            render_area: area,
            inputs: Vec::new(),
            outputs: Vec::new(),
            present_to_swapchain: false,
            record_fn: Box::new(record_fn),
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
    fn read_image_check(&mut self, image: &Handle<SampledImage>, access_type: AccessType) {
        if self.inputs.iter().any(|input| match input {
            Input::SampleImage(existing_image, _, _) | Input::ReadImage(existing_image, _) => {
                existing_image == image
            }
        }) {
            panic!("Image handle {:?} already registered for read", image);
        }

        match access_type {
            AccessType::AnyShaderReadSampledImageOrUniformTexelBuffer
            | AccessType::VertexShaderReadSampledImageOrUniformTexelBuffer
            | AccessType::TessellationControlShaderReadSampledImageOrUniformTexelBuffer
            | AccessType::TessellationEvaluationShaderReadSampledImageOrUniformTexelBuffer
            | AccessType::GeometryShaderReadSampledImageOrUniformTexelBuffer
            | AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer
            | AccessType::FragmentShaderReadColorInputAttachment
            | AccessType::FragmentShaderReadDepthStencilInputAttachment => {}
            _ => panic!(
                "Invalid access type {:?}, for renderpass sample_image",
                access_type
            ),
        }
    }
    pub fn read_image(mut self, image: &Handle<SampledImage>, access_type: AccessType) -> Self {
        self.read_image_check(image, access_type);

        self.inputs
            .push(Input::ReadImage(image.clone(), access_type));
        self
    }
    /// Used to add an "input" image to this pass, which will be automatically bound at the specified binding and be available in shaders for sampling
    pub fn sample_image(
        mut self,
        image: &Handle<SampledImage>,
        access_type: AccessType,
        binding: u32,
    ) -> Self {
        self.read_image_check(image, access_type);

        self.inputs
            .push(Input::SampleImage(image.clone(), access_type, binding));
        self
    }
    // pub fn read_image(mut self, image: &GpuHandle<SampledImage>, access_type: AccessType) -> Self {
    //     if self.inputs.iter().any(|input| {
    //         match input {
    //             Input::ReadImage(existing_image, _) => existing_image == image,
    //             Input::SampleImage(existing_image, _, _) => existing_image == image,
    //             _=> false
    //         }
    //     }) {
    //         panic!("Image handle {:?} already registered for read", image);
    //     }

    //     match access_type {
    //         AccessType::VertexShaderReadSampledImageOrUniformTexelBuffer |
    //         AccessType::VertexShaderReadOther |
    //         AccessType::TessellationControlShaderReadSampledImageOrUniformTexelBuffer |
    //         AccessType::TessellationControlShaderReadOther |
    //         AccessType::TessellationEvaluationShaderReadSampledImageOrUniformTexelBuffer |
    //         AccessType::TessellationEvaluationShaderReadOther |
    //         AccessType::GeometryShaderReadSampledImageOrUniformTexelBuffer |
    //         AccessType::GeometryShaderReadOther |
    //         AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer |
    //         AccessType::FragmentShaderReadColorInputAttachment |
    //         AccessType::FragmentShaderReadDepthStencilInputAttachment |
    //         AccessType::FragmentShaderReadOther |
    //         AccessType::ColorAttachmentRead |
    //         AccessType::DepthStencilAttachmentRead |
    //         AccessType::TransferRead |
    //         AccessType::HostRead => {},
    //         _=> panic!("Invalid access type {:?}, for renderpass read_image", access_type)

    //     }

    //     self.inputs.push(Input::ReadImage(image.clone(), access_type));
    //     self
    // }
    // pub fn write_image(mut self, image: &GpuHandle<SampledImage>, access_type: AccessType) -> Self {
    //     if self.outputs.iter().any(|output| {
    //         match output {
    //             Output::WriteImage(existing_image, _) |
    //             Output::DrawImage(existing_image, _, _) => existing_image == image,
    //             _=> false
    //         }
    //     }) {
    //         panic!("Image handle {:?} already registered for writes", image);
    //     }

    //     match access_type {
    //         AccessType::VertexShaderWrite |
    //         AccessType::TessellationControlShaderWrite |
    //         AccessType::TessellationEvaluationShaderWrite |
    //         AccessType::GeometryShaderWrite |
    //         AccessType::FragmentShaderWrite |
    //         AccessType::ColorAttachmentWrite |
    //         AccessType::DepthStencilAttachmentWrite |
    //         AccessType::DepthAttachmentWriteStencilReadOnly |
    //         AccessType::StencilAttachmentWriteDepthReadOnly |
    //         AccessType::HostWrite |
    //         AccessType::ColorAttachmentReadWrite |
    //         AccessType::General => {},
    //         _=> panic!("Invalid access type {:?}, for renderpass write_image", access_type)
    //     }

    //     self.outputs.push(Output::WriteImage(image.clone(), access_type));
    //     self
    // }

    /// Draws to the particular image as a render attachment when the renderpass is executed
    pub fn draw_image(
        mut self,
        image: &Handle<SampledImage>,
        attachment_config: AttachmentConfig,
    ) -> Self {
        // if self.outputs.iter().any(|output| match output {
        //     Output::WriteImage(existing_image, _) | Output::DrawImage(existing_image, _) => {
        //         existing_image == image
        //     }
        //     _ => false,
        // }) {
        //     panic!("Image handle {:?} already registered for writes", image);
        // }

        self.outputs
            .push(Output::DrawImage(image.clone(), attachment_config));
        self
    }

    /// Marks that the renderpass will be used for presentation to the swapchain.
    /// If a Renderpass has been marked for presentation, draws to other images is not permitted, and only the Swapchain's Framebuffer
    /// consisting of a single Color Attachment(Binding 0) and a Depth Stencil Attachment(Binding 1) will be available
    pub fn present(mut self) -> Self {
        self.present_to_swapchain = true;

        if self
            .outputs
            .iter()
            .find(|output| match output {
                Output::DrawImage(_, _) => true,
                _ => false,
            })
            .is_some()
        {
            panic!("Renderpass has been marked for presentation, draws to other images is not permitted");
        }

        self
    }
}
