use hikari_render::*;
use std::path::*;

use std::sync::Arc;

#[derive(PartialEq, Eq, Hash)]
struct ShaderKeyBorrowed<'a> {
    name: &'a str,
    defines: &'a[&'a str]
}
impl<'a> ShaderKeyBorrowed<'a> {
    fn get_hash(&self) -> u64 {
        hikari_utils::hash::quick_hash((self.name, self.defines))
    }
}

#[derive(PartialEq, Eq)]
struct ShaderInfo {
    name: String,
    defines: Vec<String>,
    shader: Arc<Shader>,
}

impl ShaderInfo {
    pub fn new(name: String, defines: Vec<String>, shader: Arc<Shader>) -> Self {
        Self {
            name,
            defines,
            shader,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn defines(&self) -> &[String] {
        &self.defines
    }
    pub fn shader(&self) -> &Arc<Shader> {
        &self.shader
    }
    pub fn shader_mut(&mut self) -> &mut Arc<Shader> {
        &mut self.shader
    }
    pub fn get_hash(&self) -> u64 {
        hikari_utils::hash::quick_hash((&self.name, &self.defines))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShaderId(u64);

#[derive(Clone, Copy)]
pub struct ShaderLibraryConfig {
    pub generate_debug_info: bool,
}
pub struct ShaderLibrary {
    device: Arc<Device>,
    shaders: hikari_utils::hash::NoHashMap<u64, ShaderInfo>,
    shader_dir: PathBuf,
    config: ShaderLibraryConfig,
}

impl ShaderLibrary {
    pub fn new(
        device: &Arc<Device>,
        shader_dir: impl AsRef<Path>,
        config: ShaderLibraryConfig,
    ) -> Self {
        assert!(shader_dir.as_ref().is_dir());
        Self {
            device: device.clone(),
            shaders: Default::default(),
            shader_dir: shader_dir.as_ref().canonicalize().unwrap().to_owned(),
            config,
        }
    }
    pub fn insert(&mut self, name: &str) -> anyhow::Result<ShaderId> {
        self.insert_with_defines(name, &[])
    }
    pub fn insert_with_defines(&mut self, name: &str, defines: &[&str]) -> anyhow::Result<ShaderId> {
        let defines: Vec<String> = defines.iter().map(|&define| define.to_owned()).collect();

        let shader = Self::create_shader(&self.device, &self.shader_dir, name, &defines, self.config)?;
        let shader_info = ShaderInfo::new(name.to_owned(), defines, shader);
        let hash = shader_info.get_hash();
        self.shaders.insert(hash, shader_info);
        Ok(ShaderId(hash))
    }
    pub fn config(&self) -> &ShaderLibraryConfig {
        &self.config
    }
    pub fn set_generate_debug(&mut self, config: ShaderLibraryConfig) -> anyhow::Result<()> {
        self.config = config;
        self.reload()
    }
    fn create_shader(
        device: &Arc<Device>,
        shader_dir: &Path,
        name: &str,
        global_defines: &[String],
        config: ShaderLibraryConfig,
    ) -> anyhow::Result<Arc<Shader>> {
        let mut path = shader_dir.join(name);
        let stage_exts = [
            (ShaderStage::Vertex, ["HK_VERTEX_SHADER"], "vert"),
            (ShaderStage::Fragment, ["HK_FRAGMENT_SHADER"], "frag"),
            (ShaderStage::TessControl, ["HK_TESS_CONTROL_SHADER"], "tesc"),
            (ShaderStage::TessEvaluation, ["HK_TESS_EVAL_SHADER"], "tese"),
            (ShaderStage::Geometry, ["HK_GEOMETRY_SHADER"], "geom"),
            (ShaderStage::Compute, ["HK_COMPUTE_SHADER"], "comp"),
        ];

        let filename = path.file_name().unwrap().to_owned();

        let mut compile_options = shaderc::CompileOptions::new().unwrap();

        if config.generate_debug_info {
            compile_options.set_generate_debug_info();
        }

        compile_options.set_include_callback(
            move |requested_source, ty, _requestee, _depth| -> shaderc::IncludeCallbackResult {
                match ty {
                    shaderc::IncludeType::Standard => {
                        let include_path = shader_dir.join(requested_source);
                        match std::fs::read_to_string(include_path) {
                            Ok(content) => shaderc::IncludeCallbackResult::Ok({
                                shaderc::ResolvedInclude {
                                    resolved_name: requested_source.to_owned(),
                                    content,
                                }
                            }),
                            Err(why) => shaderc::IncludeCallbackResult::Err(why.to_string()),
                        }
                    }
                    shaderc::IncludeType::Relative => todo!(),
                }
            },
        );

        let filename = filename.to_str().unwrap();
        let mut shader_builder = Shader::builder(&filename);

        let mut global_defines_ref: Vec<&str> = Vec::new();
        for define in global_defines {
            global_defines_ref.push(define);
        } 

        let mut atleast_one_stage = false;

        for (stage, stage_defines, ext) in &stage_exts {
            let mut defines = global_defines_ref.clone();
            for define in stage_defines {
                defines.push(define);
            }

            path.set_extension(ext);
            if path.exists() {
                let source_text = std::fs::read_to_string(&path)?;
                let code = ShaderCode {
                    entry_point: "main",
                    data: ShaderData::Glsl(source_text),
                };

                shader_builder = shader_builder.with_stage(*stage, code, &defines);
                atleast_one_stage = true;
            }
        }

        if !atleast_one_stage {
            return Err(anyhow::anyhow!("No shader stages found at path, make sure to put suitable extensions for each stage"));
        }

        Ok(shader_builder.build(device, Some(compile_options))?)
    }
    pub fn reload(&mut self) -> anyhow::Result<()> {
        for (_, shader_info) in self.shaders.iter_mut() {
            let shader = Self::create_shader(&self.device, &self.shader_dir, shader_info.name(), shader_info.defines(), self.config)?;
            *shader_info.shader_mut() = shader;
        }

        Ok(())
    }
    pub fn get(&self, name: &str) -> Option<&Arc<Shader>> {
        self.get_with_defines(name, &[])
    }
    pub fn get_with_defines(&self, name: &str, defines: &[&str]) -> Option<&Arc<Shader>> {
        let borrowed_key = ShaderKeyBorrowed {
            name,
            defines
        };
        self.shaders.get(&borrowed_key.get_hash())
        .map(|shader_info|shader_info.shader())
    }
    pub fn get_by_id(&self, id: ShaderId) -> Option<&Arc<Shader>> {
        self.shaders.get(&id.0).map(|shader_info| shader_info.shader())
    }
}
