use editor::Editor;
use hikari::input::InputPlugin;
use hikari::*;
use winit::dpi::LogicalSize;
use winit::event::*;

use hikari::core::*;
use hikari::render::*;
use hikari::systems::*;
use winit::event_loop::ControlFlow;

use crate::editor::EditorConfig;

mod editor;

struct EditorPlugin {
    log_listener: editor::logging::LogListener,
}

pub type EditorGraph = render::Graph<()>;
fn prepare_graph(
    gfx: &mut Gfx,
    backend: &imgui_support::Backend,
    mut renderer: imgui_support::Renderer,
) -> EditorGraph {
    let draw_data = backend.shared_draw_data().clone();

    let pass = Renderpass::<()>::new(
        "Imgui",
        ImageSize::default(),
        move |cmd: &mut RenderpassCommands, ()| {
            renderer.render_from_shared(cmd.raw(), &draw_data).unwrap();
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
        let editor = Editor::new(
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

        let graph = prepare_graph(gfx.as_mut(), &backend, renderer);
        drop(gfx);

        let static_window: &'static winit::window::Window =
            unsafe { std::mem::transmute(game.window()) };

        fn imgui_update<'a>(
            imgui: &'a mut imgui_support::Backend,
            window: &winit::window::Window,
            editor: &mut Editor,
        ) {
            imgui.new_frame_shared(window, |ui| {
                editor.run(ui);
            });
        }
        game.add_state(backend);
        game.add_state(graph);
        game.add_state(static_window);
        game.add_state(editor);
        game.add_task(
            core::UPDATE,
            Task::new(
                "EditorUpdate",
                |editor: &mut Editor,
                 imgui: &mut imgui_support::Backend,
                 window: &&'static winit::window::Window| {
                    imgui_update(imgui, *window, editor);
                },
            ),
        );
        game.add_task(
            core::RENDER,
            Task::new("EditorRender", |graph: &mut EditorGraph, window: &&'static winit::window::Window| {
                let window_size = window.inner_size();
                if window_size.width == 0 || window_size.height == 0 {
                    return;
                }
                graph.execute(()).expect("Failed to render imgui");
            }),
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
    game.add_plugin(EditorPlugin { log_listener });

    game.run();
}
