use hikari_asset::AssetStorage;
use hikari_core::World;
use hikari_render::{Gfx, GraphBuilder, SampledImage};

#[cfg(feature = "editor")]
use hikari_render::{AccessType, Renderpass};

use crate::{depth_prepass, fxaa, pbr, Args, Config, Settings};

pub struct WorldRenderer {
    graph: hikari_render::Graph<Args>,
    config: Config 
}

impl WorldRenderer {
    pub fn new(gfx: &mut Gfx, width: u32, height: u32) -> anyhow::Result<Self> {
        let device = gfx.device().clone();
        let mut graph = GraphBuilder::<Args>::new(gfx, width, height);
        let depth_prepass = depth_prepass::build_pass(&device, &mut graph)?;
        let pbr_output = pbr::build_pass(&device, &mut graph, &depth_prepass)?;
        let _fxaa_output = fxaa::build_pass(&device, &mut graph, &pbr_output)?;

        #[cfg(feature = "editor")]
        graph.add_renderpass(Renderpass::empty("DummyTransitionForImGui").read_image(
            &_fxaa_output,
            AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
        ));

        Ok(Self {
            graph: graph.build()?,
            config: Config {
            settings: Settings::new(),
            viewport: (width, height)
            }
        })
    }
    pub fn settings(&mut self) -> &mut Settings {
        &mut self.config.settings
    }
    pub fn get_output_image(&self) -> &SampledImage {
        self.graph
            .resources()
            .get_image_by_name("FXAAOutput")
            .unwrap()
    }
    pub fn render(&mut self, world: &World, asset_storage: &AssetStorage) -> anyhow::Result<()> {
        hikari_dev::profile_function!();
        self.graph.execute((world, &self.config, asset_storage))?;

        Ok(())
    }
    pub fn render_sync(
        &mut self,
        world: &World,
        asset_storage: &AssetStorage,
    ) -> anyhow::Result<&SampledImage> {
        hikari_dev::profile_function!();
        self.graph.execute_sync((world, &self.config, asset_storage))?;

        Ok(self.get_output_image())
    }
    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.config.viewport = (width, height);
    }
    pub fn viewport(&self) -> (u32, u32) {
        self.config.viewport
    }
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.graph.resize(width, height)
    }
    pub fn resize_and_set_viewport(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.set_viewport(width, height);
        self.resize(width, height)
    }   
    pub fn size(&self) -> (u32, u32) {
        self.graph.size()
    }
    pub fn prepare_exit(&mut self) {
        self.graph.prepare_exit()
    }
}
