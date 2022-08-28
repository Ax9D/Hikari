use hikari_3d::ShaderLibrary;
use hikari_asset::AssetStorage;
use hikari_core::{Plugin, World};

mod passes;
mod util;
mod world_renderer;
mod world;
mod light;
mod resources;
mod packet;

use hikari_render::Gfx;
use resources::RenderResources;
#[cfg(not(feature = "editor"))]
use winit::event::{Event, WindowEvent};
use world::*;
use light::*;
pub use resources::*;
pub use world_renderer::WorldRenderer;

type Args = (World, RenderResources, ShaderLibrary, AssetStorage);

#[derive(Clone, Default)]
pub struct DebugSettings {
    pub show_shadow_cascades: bool
}

#[derive(Clone)]
pub struct Settings {
    pub fxaa: bool,
    pub debug: DebugSettings
}

impl Settings {
    fn new() -> Self {
        Self { fxaa: true, debug: DebugSettings::default() }
    }
}
pub struct PBRPlugin {
    pub width: u32,
    pub height: u32,
}

impl Plugin for PBRPlugin {
    fn build(self, game: &mut hikari_core::Game) {
        let mut gfx = game.get_mut::<Gfx>();
        let mut shader_lib = game.get_mut::<ShaderLibrary>();

        let renderer = WorldRenderer::new(&mut gfx, self.width, self.height, &mut shader_lib)
            .expect("Failed to create WorldRenderer");
        drop(gfx);
        drop(shader_lib);

        game.add_state(renderer);
        #[cfg(not(feature = "editor"))]
        {
            game.add_task(
                hikari_core::RENDER,
                hikari_systems::Task::new(
                    "WorldRender",
                    move |renderer: &mut WorldRenderer,
                          world: &World,
                          shader_lib: &ShaderLibrary,
                          assets: &AssetStorage| {
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
                                .resize(size.width, size.height)
                                .expect("Failed to resize graph");
                        }
                    }
                    _ => {}
                },
                Event::LoopDestroyed => {
                    state.get_mut::<WorldRenderer>().unwrap().prepare_exit();
                }
                _ => {}
            });
        }
    }
}
