use std::sync::Arc;

use hikari::g3d::Plugin3D;
use hikari::pbr::PBRPlugin;
use hikari::render::imgui_support::Renderer;
use hikari::render::imgui_support::TextureExt;
pub use hikari_imgui as imgui;

use editor::Editor;
use hikari::input::InputPlugin;
use hikari::*;
use parking_lot::Mutex;
use winit::dpi::LogicalSize;
use winit::event::*;

use hikari::core::*;
use hikari::render::*;
use hikari::systems::*;
use winit::event_loop::ControlFlow;

use crate::editor::EditorConfig;

mod component_impls;
mod components;
mod editor;
mod widgets;

const EDITOR_STAGE: &str = "EDITOR_STAGE";
struct EditorPlugin {
    log_listener: editor::logging::LogListener,
}

pub type EditorGraph = render::Graph<()>;
fn prepare_graph(
    gfx: &mut Gfx,
    backend: &imgui_support::Backend,
    renderer: Arc<Mutex<Renderer>>,
) -> EditorGraph {
    let draw_data = backend.shared_draw_data().clone();

    let pass = Renderpass::<()>::new(
        "Imgui",
        ImageSize::default_xy(),
        move |cmd: &mut RenderpassCommands, ()| {
            renderer
                .lock()
                .render_from_shared(cmd.raw(), &draw_data)
                .unwrap();
        },
    )
    .present();

    let swapchain = gfx.swapchain().unwrap().lock();
    let (width, height) = swapchain.size();
    drop(swapchain);

    let mut gb = GraphBuilder::new(gfx, width, height);
    gb.add_renderpass(pass);

    gb.build().unwrap()
}

impl Plugin for EditorPlugin {
    fn build(self, game: &mut Game) {
        let imgui = imgui::Context::create();
        let mut backend = imgui_support::Backend::new(game.window(), imgui)
            .expect("Failed to create imgui context");
        let hidpi_factor = backend.hidpi_factor() as f32;
        Editor::init(
            game,
            backend.context(),
            EditorConfig {
                log_listener: self.log_listener,
                hidpi_factor,
            },
        );

        let mut gfx = game.get_mut::<Gfx>();
        let swapchain = gfx.swapchain().unwrap().lock();
        let color_format = swapchain.color_format();
        let depth_format = swapchain.depth_format();

        drop(swapchain);
        let renderer = imgui_support::Renderer::new(
            gfx.device(),
            &mut backend,
            color_format,
            depth_format,
            true,
        )
        .expect("Failed to create imgui renderer");
        let renderer = Arc::new(Mutex::new(renderer));
        imgui::Ui::initialize_texture_support(renderer.clone());

        let graph = prepare_graph(gfx.as_mut(), &backend, renderer);
        drop(gfx);

        let static_window: &'static winit::window::Window =
            unsafe { std::mem::transmute(game.window()) };

        game.add_state(backend);
        game.add_state(graph);
        game.add_state(static_window);

        let update_task = unsafe {
            Task::with_raw_function(
                "EditorUpdate",
                Function::from_raw(Box::new(|state| {
                    let window = *state.get::<&'static winit::window::Window>().unwrap();
                    let mut imgui = state.get_mut::<imgui_support::Backend>().expect("");
                    let mut editor = state.get_mut::<Editor>().unwrap();
                    imgui.new_frame_shared(window, |ui| {
                        editor.run(ui, state);
                    });
                })),
            )
        };
        game.add_task(POST_RENDER, update_task);
        #[allow(unused_variables)]
        game.add_task(
            POST_RENDER,
            Task::new(
                "EditorRender",
                |gfx: &Gfx, graph: &mut EditorGraph, window: &&'static winit::window::Window| {
                    hikari::dev::profile_scope!("ImGui Render");
                    let window_size = window.inner_size();
                    if window_size.width == 0 || window_size.height == 0 {
                        return;
                    }
                    let result = graph.execute(());
                    match result {
                        Ok(_) => {}
                        Err(err) => {
                            if err == vk::Result::ERROR_DEVICE_LOST {
                                log::error!("Device Lost");
                                #[cfg(feature = "aftermath")]
                                gfx.device()
                                    .wait_for_aftermath_dump()
                                    .expect("Failed to collect aftermath dump");
                                panic!();
                            }
                        }
                    }
                },
            )
            .after("EditorUpdate"),
        );

        game.add_platform_event_hook(|state, window, event, control| {
            state
                .get_mut::<imgui_support::Backend>()
                .unwrap()
                .handle_event(window, event);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        if !(size.width == 0 || size.height == 0) {
                            state
                                .get_mut::<EditorGraph>()
                                .unwrap()
                                .resize(size.width, size.height)
                                .expect("Failed to resize graph");
                        }
                    }
                    WindowEvent::CloseRequested => {
                        state.get_mut::<Editor>().unwrap().handle_exit();
                        *control = ControlFlow::Exit;
                    }
                    _ => {}
                },
                Event::LoopDestroyed => {
                    state.get_mut::<EditorGraph>().unwrap().prepare_exit();
                }
                _ => {}
            }
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_listener = editor::logging::init()?;

    let window = winit::window::WindowBuilder::new()
        .with_title("Hikari Editor")
        .with_inner_size(LogicalSize::new(1920.0, 1080.0))
        .with_resizable(true);
    let mut game = Game::new(window)?;

    game.add_plugin(CorePlugin);
    game.add_plugin(InputPlugin);

    game.add_plugin(GfxPlugin {
        config: GfxConfig {
            debug: false,
            features: Features::default(),
            vsync: true,
        },
    });

    game.add_plugin(Plugin3D);
    game.add_plugin(PBRPlugin {
        width: 1920,
        height: 1080,
    });

    game.add_plugin(EditorPlugin { log_listener });
    game.run();
}
