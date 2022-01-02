pub mod pipeline;
use vk_sync_fork::AccessType;

use crate::{
    graph::{command::RenderpassCommands, Handle},
    texture::SampledImage,
};

use super::{AttachmentConfig, ImageSize, Input, Output};

pub use pipeline::*;

pub struct Renderpass<Scene, PerFrame, Resources> {
    name: String,
    id: u64,
    pub(crate) render_area: ImageSize,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    pub(crate) present_to_swapchain: bool,
    pub(crate) draw_fn: Box<dyn FnMut(&mut RenderpassCommands, &Scene, &PerFrame, &Resources)>,
}

impl<Scene, PerFrame, Resources> Renderpass<Scene, PerFrame, Resources> {
    pub fn new(
        name: &str,
        area: ImageSize,
        draw_fn: impl FnMut(&mut RenderpassCommands, &Scene, &PerFrame, &Resources) + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            id: crate::util::quick_hash(name),
            render_area: area,
            inputs: Vec::new(),
            outputs: Vec::new(),
            present_to_swapchain: false,
            draw_fn: Box::new(draw_fn),
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
    pub fn sample_image(
        mut self,
        image: &Handle<SampledImage>,
        access_type: AccessType,
        binding: u32,
    ) -> Self {
        if self.inputs.iter().any(|input| match input {
            Input::SampleImage(existing_image, _, binding) => existing_image == image,
            _ => false,
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
