use std::sync::Arc;

use hikari_3d::*;
use hikari_asset::AssetManager;
use hikari_core::{Entity, World};
use hikari_math::*;
use hikari_render::*;

#[cfg(feature = "editor")]
use hikari_render::{AccessType, Renderpass};

use crate::{
    passes::{self},
    util,
    world::WorldUBO,
    Args, RenderResources, Settings,
};

pub struct WorldRenderer {
    graph: hikari_render::Graph<Args>,
    primitives: Arc<hikari_3d::primitives::Primitives>,
    res: RenderResources,
}

impl WorldRenderer {
    pub fn new(
        gfx: &mut Gfx,
        width: u32,
        height: u32,
        shader_library: &mut ShaderLibrary,
        primitives: &Arc<hikari_3d::primitives::Primitives>,
    ) -> anyhow::Result<Self> {
        let res = RenderResources::new(gfx.device(), width, height)?;
        let graph = Self::build_graph(gfx, width, height, shader_library, primitives, &res)?;
        Ok(Self {
            graph,
            res,
            primitives: primitives.clone(),
        })
    }
    fn build_graph(
        gfx: &mut Gfx,
        width: u32,
        height: u32,
        shader_library: &mut ShaderLibrary,
        primitives: &Arc<hikari_3d::primitives::Primitives>,
        res: &RenderResources,
    ) -> anyhow::Result<hikari_render::Graph<Args>> {
        let device = gfx.device().clone();
        let mut graph = GraphBuilder::<Args>::new(gfx, width, height);
        let depth_prepass = passes::depth_prepass::build_pass(&device, &mut graph, shader_library)?;
        let (shadow_cascades, cascade_render_buffer) = passes::shadow::build_pass(
            &device,
            &mut graph,
            shader_library,
            &res.settings,
            &depth_prepass,
        )?;
        let pbr_output = passes::pbr::build_pass(
            &device,
            &mut graph,
            shader_library,
            primitives,
            &shadow_cascades,
            &cascade_render_buffer,
            &depth_prepass,
        )?;
        #[cfg(feature = "editor")]
        let _fxaa_output =
            passes::fxaa::build_pass(&device, &mut graph, shader_library, &pbr_output)?;

        #[cfg(not(feature = "editor"))]
        passes::fxaa::build_pass(&device, &mut graph, shader_library, &pbr_output)?;

        #[cfg(feature = "editor")]
        {
            //let debug = passes::debug::build_pass(&device, &mut graph, shader_library, &depth_prepass, &shadow_cascades)?;

            let fake_pass = Renderpass::new("DummyTransitionForImGui", ImageSize::default_xy())
                .read_image(
                    &_fxaa_output,
                    AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
                );

            // for rt in debug {
            //     fake_pass = fake_pass.read_image(&rt, AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer);
            // }

            graph.add_renderpass(fake_pass);
        }

        Ok(graph.build()?)
    }
    pub fn update_settings(
        &mut self,
        gfx: &mut Gfx,
        shader_library: &mut ShaderLibrary,
        mut update_fn: impl FnMut(&mut Settings),
    ) -> anyhow::Result<()> {
        let old_settings = self.res.settings.clone();

        (update_fn)(&mut self.res.settings);

        if self.res.settings.directional_shadow_map_resolution
            != old_settings.directional_shadow_map_resolution
        {
            let (width, height) = self.graph.size();

            self.graph.finish()?;
            self.graph = Self::build_graph(
                gfx,
                width,
                height,
                shader_library,
                &self.primitives,
                &self.res,
            )?;
        }

        gfx.set_vsync(self.res.settings.vsync);

        Ok(())
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

        let mut ubo_data = WorldUBO::default();

        if let Some(entity) = camera {
            let mut query = world.query_one::<(&Transform, &Camera)>(entity).unwrap();
            let (transform, camera) = query.get().unwrap();

            let projection = camera.get_projection_matrix(res.viewport.0, res.viewport.1);
            let view = transform.get_matrix().inverse();

            let camera_view_proj = projection * view;
            ubo_data.camera_position = transform.position.into();
            ubo_data.proj = projection;
            ubo_data.view = view;
            ubo_data.view_proj = camera_view_proj;
            ubo_data.camera_near = camera.near;
            ubo_data.camera_far = camera.far;
            ubo_data.viewport_size = res.viewport.into();
            ubo_data.exposure = camera.exposure;

            if let Some((_, (transform, environment))) =
                world.query::<(&Transform, &Environment)>().iter().next()
            {
                ubo_data.environment_intensity = environment.intensity;
                ubo_data.environment_transform = transform.get_rotation_matrix();
            }

            if let Some(entity) = directional_light {
                let mut query = world.query_one::<(&Transform, &Light)>(entity).unwrap();
                let (transform, light) = query.get().unwrap();

                let direction = transform.forward();
                let up_direction = transform.up();

                ubo_data.dir_light.intensity = light.intensity;
                ubo_data.dir_light.size = light.size;
                ubo_data.dir_light.color = light.color.into();
                ubo_data.dir_light.direction = direction.into();
                ubo_data.dir_light.up_direction = up_direction.into();
                ubo_data.dir_light.cascade_split_lambda = light.shadow.cascade_split_lambda;
                ubo_data.dir_light.cast_shadows = light.shadow.enabled.into();
                ubo_data.show_cascades = res.settings.debug.show_shadow_cascades as u32;
                if light.shadow.enabled {
                    let shadow = &light.shadow;
                    ubo_data.dir_light.normal_bias = shadow.normal_bias;
                    ubo_data.dir_light.shadow_fade = shadow.fade;
                    ubo_data.dir_light.max_shadow_distance = shadow.max_shadow_distance;
                    passes::shadow::compute_cascades(&mut ubo_data, &res.settings);
                }
            }
        }

        world_ubo.mapped_slice_mut()[0] = ubo_data;
    }
    fn reset(&mut self) {
        self.res.camera = None;
        self.res.directional_light = None;
        self.res.world_ubo.new_frame();
    }
    pub fn render(
        &mut self,
        world: &World,
        shader_lib: &ShaderLibrary,
        asset_manager: &AssetManager,
    ) -> anyhow::Result<()> {
        hikari_dev::profile_function!();

        self.prepare(world, None);
        self.graph
            .execute((world, &self.res, shader_lib, asset_manager))?;

        self.reset();
        Ok(())
    }
    #[cfg(feature = "editor")]
    pub fn render_editor(
        &mut self,
        world: &World,
        camera: Option<hikari_core::Entity>,
        shader_lib: &ShaderLibrary,
        asset_manager: &AssetManager,
    ) -> anyhow::Result<&SampledImage> {
        hikari_dev::profile_function!();

        self.prepare(world, camera);
        self.graph
            .execute((world, &self.res, shader_lib, asset_manager))?;

        self.reset();
        Ok(self.get_output_image())
    }
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.res.viewport = (width, height);
    }
    pub fn viewport(&self) -> (f32, f32) {
        self.res.viewport
    }
    pub fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        self.graph.resize(width, height)?;

        self.res.on_resize(width, height)
    }
    pub fn resize_and_set_viewport(&mut self, width: f32, height: f32) -> anyhow::Result<()> {
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
