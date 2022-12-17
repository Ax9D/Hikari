use hikari_3d::ShaderLibrary;
use hikari_asset::AssetManager;
use hikari_core::{Plugin, World};

mod light;
mod packet;
mod passes;
mod resources;
mod util;
mod world;
mod world_renderer;

use hikari_render::Gfx;
use light::*;
use resources::RenderResources;
pub use resources::*;
#[cfg(not(feature = "editor"))]
use winit::event::{Event, WindowEvent};
use world::*;
pub use world_renderer::WorldRenderer;

type Args = (World, RenderResources, ShaderLibrary, AssetManager);

#[derive(Clone, Default, PartialEq, Eq)]
pub struct DebugSettings {
    pub show_shadow_cascades: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShadowResolution {
    D256 = 0,
    D512,
    D1024,
    D2048,
    D4096,
}

impl ShadowResolution {
    pub fn size(self) -> u32 {
        match self {
            ShadowResolution::D256 => 256,
            ShadowResolution::D512 => 512,
            ShadowResolution::D1024 => 1024,
            ShadowResolution::D2048 => 2048,
            ShadowResolution::D4096 => 4096,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Settings {
    pub fxaa: bool,
    pub vsync: bool,
    pub directional_shadow_map_resolution: ShadowResolution,
    pub debug: DebugSettings,
}

impl Settings {
    fn new() -> Self {
        Self {
            fxaa: true,
            vsync: true,
            directional_shadow_map_resolution: ShadowResolution::D2048,
            debug: DebugSettings::default(),
        }
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
                          assets: &AssetManager| {
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
                Event::LoopDestroyed => {
                    state.get_mut::<WorldRenderer>().unwrap().prepare_exit();
                }
                _ => {}
            });
        }
    }
}
