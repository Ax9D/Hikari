use std::sync::Arc;

use hikari_asset::AssetStorage;
use hikari_core::{World, Entity};
use hikari_3d::*;
use hikari_math::*;
use hikari_render::*;

#[cfg(feature = "editor")]
use hikari_render::{AccessType, Renderpass};

use crate::{Args, RenderResources, Settings, world::WorldUBO, util, passes};

pub struct WorldRenderer {
    graph: hikari_render::Graph<Args>,
    res: RenderResources,
}

impl WorldRenderer {
    fn create_render_resources(device: &Arc<Device>, width: u32, height: u32) -> anyhow::Result<RenderResources> {
        log::debug!("sizeof(UBO)={}", std::mem::size_of::<WorldUBO>());

        Ok(RenderResources { 
            settings: Settings::new(),
            viewport: (width as f32, height as f32),
            camera: None,
            directional_light: None,
            world_ubo: PerFrame::new([create_uniform_buffer(device, 1)?, create_uniform_buffer(device, 1)?])
        })
    }
    pub fn new(
        gfx: &mut Gfx,
        width: u32,
        height: u32,
        shader_library: &mut ShaderLibrary,
    ) -> anyhow::Result<Self> {
        let device = gfx.device().clone();
        let mut graph = GraphBuilder::<Args>::new(gfx, width, height);
        let shadow_cascades = passes::shadow::build_pass(&device, &mut graph, shader_library)?;
        let depth_prepass = passes::depth_prepass::build_pass(&device, &mut graph, shader_library)?;
        let pbr_output = passes::pbr::build_pass(&device, &mut graph, shader_library, &shadow_cascades, &depth_prepass)?;
        let _fxaa_output = passes::fxaa::build_pass(&device, &mut graph, shader_library, &pbr_output)?;

        #[cfg(feature = "editor")]
        let (depth_debug, shadow_debug) = passes::debug::build_pass(&device, &mut graph, shader_library, &depth_prepass, &shadow_cascades)?;
        
        let mut fake_pass = Renderpass::empty("DummyTransitionForImGui")
        .read_image(
            &_fxaa_output,
            AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
        )
        .read_image(&depth_debug, AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer);

        for cascade in shadow_debug {
            fake_pass = fake_pass.read_image(&cascade, AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer);
        }
        #[cfg(feature = "editor")]
        graph.add_renderpass(
            fake_pass
        );
        // .read_image(&shadow_debug, AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer)
        // );

        Ok(Self {
            graph: graph.build()?,
            res: Self::create_render_resources(gfx.device(), width, height)?
        })
    }
    pub fn settings(&mut self) -> &mut Settings {
        &mut self.res.settings
    }
    pub fn graph_resources(&self) -> &GraphResources {
        &self.graph.resources()
    }
    pub fn get_output_image(&self) -> &SampledImage {
        self.graph
            .resources()
            .get_image_by_name("FXAAOutput")
            .unwrap()
    }
    fn prepare(&mut self, world: &World, camera: Option<Entity>) {
        let camera = camera.or(util::get_camera(world));
        let directional_light = util::get_directional_light(world);

        let res = &mut self.res;
        res.camera = camera;
        res.directional_light = directional_light;

        let world_ubo = &mut res.world_ubo;
        let world_ubo = world_ubo.get_mut();

        let mut ubo_data = WorldUBO::default();

        if let Some(entity) = camera {
            let mut query = world.query_one::<(&Transform, &Camera)>(entity).unwrap();
            let (transform, camera) = query.get().unwrap();

            let projection = camera.get_projection_matrix(res.viewport.0, res.viewport.1);
            let view = transform.get_matrix().inverse();

            let camera_view_proj = projection * view;
            ubo_data.camera_position = transform.position.into();
            ubo_data.view = view.to_cols_array();
            ubo_data.view_proj = camera_view_proj.to_cols_array();
            ubo_data.exposure = camera.exposure;

            if let Some(entity) = directional_light {
                let mut query = world.query_one::<(&Transform, &Light)>(entity).unwrap();
                let (transform, light) = query.get().unwrap();

                let direction = transform.forward();

                ubo_data.dir_light.intensity = light.intensity;
                ubo_data.dir_light.size = light.size;
                ubo_data.dir_light.color = light.color.into();
                ubo_data.dir_light.direction = direction.into();
                ubo_data.show_cascades = res.settings.debug.show_shadow_cascades as u32;
                if let Some(shadow) = light.shadow {
                    ubo_data.dir_light.constant_bias_factor = shadow.constant_bias;
                    ubo_data.dir_light.normal_bias_factor = shadow.normal_bias;
                    ubo_data.dir_light.shadow_fade = shadow.fade;
                    ubo_data.dir_light.max_shadow_distance = shadow.max_shadow_distance;
                    passes::shadow::compute_cascades(passes::shadow::MAX_SHADOW_CASCADES,
                        &shadow,
                        transform, 
                        &camera,
                        &camera_view_proj, 
                        &mut ubo_data);
                    }
            }
        }


        world_ubo.mapped_slice_mut()[0] = ubo_data;
    }
    fn reset(&mut self) {
        self.res.camera = None;
        self.res.directional_light = None;
        self.res.world_ubo.next_frame();
    }
    pub fn render(
        &mut self,
        world: &World,
        shader_lib: &ShaderLibrary,
        asset_storage: &AssetStorage,
    ) -> anyhow::Result<()> {
        hikari_dev::profile_function!();

        self.prepare(world, None);
        self.graph
            .execute((world, &self.res, shader_lib, asset_storage))?;

        self.reset();
        Ok(())
    }
    #[cfg(feature = "editor")]
    pub fn render_editor(
        &mut self,
        world: &World,
        camera: Option<hikari_core::Entity>,
        shader_lib: &ShaderLibrary,
        asset_storage: &AssetStorage,
    ) -> anyhow::Result<&SampledImage> {
        hikari_dev::profile_function!();

        self.prepare(world, camera);
        self.graph
            .execute((world, &self.res, shader_lib, asset_storage))?;

        self.reset();
        Ok(self.get_output_image())
    }
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.res.viewport = (width, height);
    }
    pub fn viewport(&self) -> (f32, f32) {
        self.res.viewport
    }
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.graph.resize(width, height)
    }
    pub fn resize_and_set_viewport(
        &mut self,
        width: f32,
        height: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.set_viewport(width, height);
        self.resize(width.round() as u32, height.round() as u32)
    }
    pub fn size(&self) -> (u32, u32) {
        self.graph.size()
    }
    pub fn prepare_exit(&mut self) {
        self.graph.prepare_exit()
    }
}
