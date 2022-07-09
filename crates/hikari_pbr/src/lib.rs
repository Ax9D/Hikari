use hikari_asset::AssetStorage;
use hikari_core::{World, Plugin};

mod depth_prepass;
mod fxaa;
mod pbr;
mod util;
mod world_renderer;

use hikari_render::Gfx;
#[cfg(not(feature = "editor"))]
use winit::event::{WindowEvent, Event};
pub use world_renderer::*;

type Args = (World, Config, AssetStorage);

pub struct Config {
    settings: Settings,
    width: u32,
    height: u32
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
pub struct PBRPlugin;

impl Plugin for PBRPlugin {
    fn build(self, game: &mut hikari_core::Game) {
        let mut gfx = game.get_mut::<Gfx>();

        let swapchain = gfx.swapchain().unwrap().lock();
        let (width, height) = swapchain.size();
        drop(swapchain);
        
        let renderer = WorldRenderer::new(&mut gfx, width, height).expect("Failed to create WorldRenderer");
        
        let device = gfx.device().clone();
        drop(gfx);

        game.add_state(renderer);
        game.add_task(hikari_core::RENDER, hikari_systems::Task::new("WorldRender", move |renderer: &mut WorldRenderer, world: &World, assets: &AssetStorage| {
            #[cfg(feature = "editor")]
            renderer.render_sync(world, assets).expect("Failed to render world");
            // if renderer.render_sync(world, assets).is_err() {
            //     device.wait_for_aftermath_dump().expect("Failed to collect aftermath dump");
            //     panic!("Failed to render world");
            // }
            #[cfg(not(feature = "editor"))]
            renderer.render(world, assets);
        }));

        #[cfg(not(feature = "editor"))]
        game.add_platform_event_hook(|state, _, event, _| {
            match event {
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
                },
                _=> {}
            }
        });
    }
}