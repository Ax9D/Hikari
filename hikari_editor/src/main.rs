use editor::Editor;
use hikari::input::InputPlugin;
use hikari::*;
use winit::dpi::LogicalSize;
use winit::event::*;

use hikari::core::*;
use hikari::render::*;
use hikari::systems::*;
use winit::event_loop::ControlFlow;

mod editor;

struct EditorPlugin;

pub type EditorGraph = render::Graph<(imgui::DrawData,)>;
fn prepare_graph(gfx: &mut Gfx, mut renderer: imgui_support::Renderer) -> EditorGraph {
    let pass = Renderpass::<(imgui::DrawData,)>::new(
        "Imgui",
        ImageSize::default(),
        move |cmd: &mut RenderpassCommands, (draw_data,)| {
            renderer.render(cmd.raw(), draw_data).unwrap();
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
        let editor = Editor::new(backend.context());

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

        let graph = prepare_graph(gfx.as_mut(), renderer);
        drop(gfx);

        let static_window: &'static winit::window::Window =
            unsafe { std::mem::transmute(game.window()) };

        fn imgui_render<'a>(
            imgui: &'a mut imgui_support::Backend,
            graph: &'a mut EditorGraph,
            window: &winit::window::Window,
            editor: &mut Editor,
        ) {
            let draw_data = imgui.new_frame(window, |ui| {
                editor.run(ui);
            });

            graph.execute((draw_data,)).expect("Failed to render imgui");
        }
        game.add_state(backend);
        game.add_state(graph);
        game.add_state(static_window);
        game.add_state(editor);
        game.add_task(
            core::LAST,
            Task::new(
                "EditorUpdate",
                |graph: &mut EditorGraph,
                 editor: &mut Editor,
                 imgui: &mut imgui_support::Backend,
                 window: &&'static winit::window::Window| {
                    imgui_render(imgui, graph, *window, editor);
                },
            ),
        );

        game.add_platform_event_hook(|state, window, event, control| {
            state
                .get_mut::<imgui_support::Backend>()
                .unwrap()
                .handle_event(window, event);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        state
                            .get_mut::<EditorGraph>()
                            .unwrap()
                            .resize(size.width, size.height)
                            .expect("Failed to resize graph");
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
    if simple_logger::SimpleLogger::new().init().is_err() {
        println!("Failed to init logger");
    }

    let window = winit::window::WindowBuilder::new()
        .with_title("Hikari Editor")
        .with_inner_size(LogicalSize::new(1920.0, 1080.0))
        .with_resizable(true);
    let mut game = Game::new(window)?;

    game.add_plugin(CorePlugin);
    game.add_plugin(InputPlugin);

    game.add_plugin(GfxPlugin {
        config: GfxConfig {
            debug: true,
            features: Features::default(),
            vsync: true,
        },
    });
    game.add_plugin(EditorPlugin);

    game.run();
}
