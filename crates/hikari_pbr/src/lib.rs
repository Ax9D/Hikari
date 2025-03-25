use std::sync::Arc;

use hikari_3d::ShaderLibrary;
use hikari_asset::AssetManager;
use hikari_core::{Plugin, World};

mod light;
mod packet;
mod passes;
mod resources;
mod util;
mod common;
mod world_renderer;

mod settings;
mod instancing;

use hikari_render::Gfx;
use light::*;
pub use resources::*;
#[cfg(not(feature = "editor"))]
use winit::event::{Event, WindowEvent};
use common::*;
pub use world_renderer::WorldRenderer;
pub use settings::*;

type Args = (World, RenderResources, ShaderLibrary, AssetManager);


pub struct PBRPlugin {
    pub width: u32,
    pub height: u32,
    pub settings: Settings,
}

impl Plugin for PBRPlugin {
    fn build(self, game: &mut hikari_core::Game) {
        let mut gfx = game.get_mut::<Gfx>();
        let mut shader_lib = game.get_mut::<ShaderLibrary>();
        let primitives = game.get::<Arc<hikari_3d::primitives::Primitives>>();
        let renderer = WorldRenderer::new_with_settings(
            &mut gfx,
            self.width,
            self.height,
            self.settings,
            &mut shader_lib,
            &primitives,
        )
        .expect("Failed to create WorldRenderer");
        drop(gfx);
        drop(shader_lib);
        drop(primitives);

        game.add_state(renderer);
        game.create_exit_stage("RendererExit");
        game.add_exit_task("RendererExit", hikari_core::Task::new("Graph Exit", |renderer: &mut WorldRenderer| {
            renderer.prepare_exit();
        }));
        
        #[cfg(not(feature = "editor"))]
        {
            game.add_task(
                hikari_core::RENDER,
                hikari_systems::Task::new(
                    "WorldRender",
                    move |renderer: &mut WorldRenderer,
                          world: &World,
                          shader_lib: &ShaderLibrary,
                          assets: &AssetManager,
                          window: &winit::window::Window| {
                        let window_size = window.inner_size();
                        if window_size.width == 0 || window_size.height == 0 {
                                return;
                        }
                        renderer
                                .render(world, shader_lib, assets)
                                .expect("Failed to render world");
                    },
                ),
            );
            game.add_platform_event_hook(|state, _, event, _| match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        if !(size.width == 0 || size.height == 0) {
                            state
                                .get_mut::<WorldRenderer>()
                                .unwrap()
                                .resize_and_set_viewport(size.width as f32, size.height as f32)
                                .expect("Failed to resize graph");
                        }
                    }
                    _ => {}
                },
                _ => {}
            });
        }
    }
}
