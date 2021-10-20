use std::{collections::HashSet, sync::Arc};

use indexmap::IndexMap;

pub mod pipeline;
pub use pipeline::Pipeline;

use crate::graph::CommandBuffer;

use super::{ColorFormat, DepthStencilFormat, ImageSize, Input, Output};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderpassValidationError {
    #[error("In pass {0:?}: cyclic dependency detected!")]
    CyclicDependency(String),
    #[error("In pass {0:?}: Uniform {1:?} corresponding to color input {2:?}  was not found in fragment shader of: {3:?}!")]
    ShaderUniformNotFound(String, String, String, String),
    #[error("In pass {0:?}: Shader output {1:?} corresponding to color output {2:?} was not found in fragment shader of: {3:?}! Maybe it was optimized out?")]
    ShaderOutputNotFound(String, String, String, String),
    #[error("In pass {0:?}: No outputs provided!")]
    NoOutputs(String),
}

#[derive(Clone)]
pub struct ColorOutput {
    pub format: ColorFormat,
    pub clear: bool,
}
#[derive(Clone)]
pub struct DepthStencilOutput {
    pub format: DepthStencilFormat,
    pub depth_clear: bool,
    pub stencil_clear: bool,
}

pub struct RenderpassBuilder<Scene, PerFrame, Resources> {
    pass: Renderpass<Scene, PerFrame, Resources>,
}
impl<Scene, PerFrame, Resources> RenderpassBuilder<Scene, PerFrame, Resources> {
    pub fn new<D: 'static + Fn(&mut CommandBuffer, &Scene, &PerFrame, &Resources)>(
        name: impl AsRef<str>,
        size: ImageSize,
        draw_fn: D,
    ) -> Self {
        Self {
            pass: Renderpass {
                name: name.as_ref().to_string(),
                inputs: IndexMap::new(),
                outputs: IndexMap::new(),
                draw_fn: Box::new(draw_fn),
                pipelines: HashSet::new(),
                size,
                is_final: false,
                outputs_swapchain: false,
            },
        }
    }
    pub fn input(mut self, name: impl AsRef<str>) -> Self {
        self.pass.inputs.insert(
            name.as_ref().to_string(),
            Input::Read(self.pass.inputs.len() as u32),
        );

        self
    }
    pub fn dependency(mut self, name: impl AsRef<str>) -> Self {
        self.pass
            .inputs
            .insert(name.as_ref().to_string(), Input::Dependency);

        self
    }
    pub fn color_output(mut self, name: impl AsRef<str>, output: ColorOutput) -> Self {
        self.pass
            .outputs
            .insert(name.as_ref().to_string(), Output::Color(output));

        self
    }
    pub fn depth_stencil_output(
        mut self,
        name: impl AsRef<str>,
        output: DepthStencilOutput,
    ) -> Self {
        self.pass
            .outputs
            .insert(name.as_ref().to_string(), Output::DepthStencil(output));

        self
    }
    pub fn will_use_pipeline(mut self, pipeline: &Arc<Pipeline>) -> Self {
        self.pass.pipelines.insert(pipeline.clone());

        self
    }
    pub fn mark_final(mut self) -> Self {
        self.pass.is_final = true;

        self
    }
    pub fn outputs_swapchain(mut self) -> Self {
        self.pass.outputs_swapchain = true;

        self
    }

    fn check_cyclic_deps(self) -> Result<Self, RenderpassValidationError> {
        let mut names = HashSet::new();

        let i_count = self.pass.inputs.len();
        let o_count = self.pass.outputs.len();

        for (input, _) in &self.pass.inputs {
            names.insert(input.clone());
        }

        for (output, _) in &self.pass.outputs {
            names.insert(output.clone());
        }

        if names.len() < i_count + o_count {
            return Err(RenderpassValidationError::CyclicDependency(self.pass.name));
        }

        Ok(self)
    }

    fn sanity_checks(self) -> Result<Self, RenderpassValidationError> {
        if self.pass.outputs.is_empty() && !self.pass.outputs_swapchain {
            return Err(RenderpassValidationError::NoOutputs(self.pass.name));
        }

        Ok(self)
    }
    fn validate(self) -> Result<Self, RenderpassValidationError> {
        Ok(self.sanity_checks()?.check_cyclic_deps()?)
    }
    pub fn build(
        self,
    ) -> Result<Renderpass<Scene, PerFrame, Resources>, Box<dyn std::error::Error>> {
        if self.pass.outputs_swapchain {
            self.pass.is_final = true;
            self.pass.outputs.clear();
        }

        Ok(self.validate()?.pass)
    }
}

pub struct Renderpass<Scene, PerFrame, Resources> {
    name: String,
    inputs: IndexMap<String, Input>,
    outputs: IndexMap<String, Output>,
    size: ImageSize,
    pub(crate) pipelines: HashSet<Arc<Pipeline>>,
    is_final: bool,
    outputs_swapchain: bool,
    pub(crate) draw_fn:
        Box<dyn Fn(&mut crate::graph::CommandBuffer, &Scene, &PerFrame, &Resources)>,
}

impl<Scene, PerFrame, Resources> Renderpass<Scene, PerFrame, Resources> {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn inputs(&self) -> &IndexMap<String, Input> {
        &self.inputs
    }
    pub fn outputs(&self) -> &IndexMap<String, Output> {
        &self.outputs
    }
    pub fn is_final(&self) -> bool {
        self.is_final
    }
    pub fn outputs_swapchain(&self) -> bool {
        self.outputs_swapchain
    }
    pub fn size(&self) -> &ImageSize {
        &self.size
    }
}
