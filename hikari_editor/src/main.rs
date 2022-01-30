use hikari::*;
use winit::event::*;
use winit::event_loop::ControlFlow;

use hikari::core::*;
use hikari::render::*;
use hikari::systems::*;

struct Editor {}

pub type EditorGraph = render::Graph<(), (), ()>;
fn prepare_graph(mut gfx: RefMut<Gfx>) -> EditorGraph {
    let pass =
    Renderpass::<(), (), ()>::new("Test", ImageSize::default(), |_, _, _, _| {}).present();

    let swapchain = gfx.swapchain().lock();
    let (width, height) = swapchain.size();
    drop(swapchain);

    let mut gb = GraphBuilder::new(&mut gfx, width, height);
    gb.add_renderpass(pass);
    gb.build().unwrap()
}

impl Plugin for Editor {
    fn build(&mut self, game: &mut Game) {
        let gfx = game.get_mut::<Gfx>();

        let graph = prepare_graph(gfx);
        game.add_state(graph);


        game.add_function(Stage::new("EditorUpdate").add_function(
            |mut gfx: RefMut<render::Gfx>, mut graph: RefMut<EditorGraph>| {
                graph
                    .execute(&mut gfx, &(), &(), &())
                    .expect("Failed to run graph");
            },
        ));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = winit::window::WindowBuilder::new()
        .with_title("Hikari Editor")
        .with_resizable(true);
    let mut game = Game::new();
    let gameloop = GameLoop::new(window)?;

    game.add_plugin(Editor {});

    gameloop.run(game, |state, _, event, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::Resized(size) => unsafe {
                state
                    .get_mut::<EditorGraph>()
                    .unwrap()
                    .resize(size.width, size.height)
                    .expect("Failed to resize graph");
            },
            _ => {}
        },
        _ => {}
    })
}
