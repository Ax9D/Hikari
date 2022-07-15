use hikari_asset::AssetStorage;
use hikari_core::{Plugin, World};

mod depth_prepass;
mod fxaa;
mod pbr;
mod util;
mod world_renderer;

use hikari_render::Gfx;
#[cfg(not(feature = "editor"))]
use winit::event::{Event, WindowEvent};
pub use world_renderer::*;

type Args = (World, Config, AssetStorage);

pub struct Config {
    settings: Settings,
    width: u32,
    height: u32,
}

#[derive(Clone)]
pub struct Settings {
    pub fxaa: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { fxaa: true }
    }
}
pub struct PBRPlugin {
    pub width: u32, 
    pub height: u32
}

impl Plugin for PBRPlugin {
    fn build(self, game: &mut hikari_core::Game) {
        let mut gfx = game.get_mut::<Gfx>();

        let renderer =
            WorldRenderer::new(&mut gfx, self.width, self.height).expect("Failed to create WorldRenderer");
        drop(gfx);
        

        game.add_state(renderer);
        game.add_task(
            hikari_core::RENDER,
            hikari_systems::Task::new(
                "WorldRender",
                move |renderer: &mut WorldRenderer, world: &World, assets: &AssetStorage| {
                    #[cfg(feature = "editor")]
                    renderer
                        .render_sync(world, assets)
                        .expect("Failed to render world");
                    // if renderer.render_sync(world, assets).is_err() {
                    //     device.wait_for_aftermath_dump().expect("Failed to collect aftermath dump");
                    //     panic!("Failed to render world");
                    // }
                    #[cfg(not(feature = "editor"))]
                    renderer.render(world, assets);
                },
            ),
        );

        #[cfg(not(feature = "editor"))]
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
