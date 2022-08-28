use std::path::*;
use hikari_render::*;
use hikari_render::shaderc;

use std::sync::Arc;
use std::collections::HashMap;

pub struct ShaderLibrary {
    device: Arc<Device>,
    shaders: HashMap<String, Arc<Shader>>,
    shader_dir: PathBuf,
}

impl ShaderLibrary {
    pub fn new(device: &Arc<Device>, shader_dir: impl AsRef<Path>) -> Self {
        assert!(shader_dir.as_ref().is_dir());
        Self { device: device.clone(), shaders: HashMap::new(), shader_dir: shader_dir.as_ref().canonicalize().unwrap().to_owned() }
    }
    pub fn insert(&mut self, name: &str) -> anyhow::Result<()> {
        let shader = Self::create_shader(&self.device, &self.shader_dir, name)?;
        self.shaders.insert(name.to_owned(), shader);
        Ok(())
    }
    fn create_shader(device: &Arc<Device>, shader_dir: &Path, name: &str) -> anyhow::Result<Arc<Shader>> {
        let mut path = shader_dir.join(name);
        let stage_exts = [
        (ShaderStage::Vertex, "vert"), 
        (ShaderStage::Fragment, "frag"), 
        (ShaderStage::TessControl, "tesc"), 
        (ShaderStage::TessEvaluation, "tese"),
        (ShaderStage::Geometry, "geom")];

        let filename = path.file_name().unwrap().to_owned();

        let mut compile_options = shaderc::CompileOptions::new().unwrap();

        compile_options.set_include_callback(move |requested_source, ty, _requestee, _depth| -> shaderc::IncludeCallbackResult {
            match ty {
                shaderc::IncludeType::Standard => {
                    let include_path = shader_dir.join(requested_source);
                    match std::fs::read_to_string(include_path) {
                        Ok(content) => shaderc::IncludeCallbackResult::Ok({
                            shaderc::ResolvedInclude {
                                resolved_name: requested_source.to_owned(),
                                content
                            }
                        }),
                        Err(why) => shaderc::IncludeCallbackResult::Err(why.to_string()),
                    }
                },
                shaderc::IncludeType::Relative => todo!(),
            }
        });

        let mut shader_builder = Shader::builder(filename.to_str().unwrap())
        .with_options(compile_options);

        let mut atleast_one_stage = false;

        for (stage, ext) in stage_exts {
            path.set_extension(ext);
            if path.exists() {
               let source_text = std::fs::read_to_string(&path)?;
               let code = ShaderCode {
                entry_point: "main",
                data: ShaderData::Glsl(source_text)
               };

               shader_builder = shader_builder.with_stage(stage, code);
               atleast_one_stage = true;
            }
        }

        if !atleast_one_stage {
            return Err(anyhow::anyhow!("No shader stages found at path, make sure to put suitable extensions for each stage"));
        }
        
        Ok(shader_builder.build(device)?)
    }
    pub fn reload(&mut self) -> anyhow::Result<()> {

        for (name, shader) in self.shaders.iter_mut() {
            *shader = Self::create_shader(&self.device, &self.shader_dir, name)?;
        }

        Ok(())
    }
    pub fn get(&self, name: &str) -> Option<&Arc<Shader>> {
        self.shaders.get(name)
    }
}