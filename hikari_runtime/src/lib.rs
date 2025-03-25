use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use hikari::asset::*;
use hikari::core::*;
use hikari::core::load_save::WorldLoaderPlugin;
use hikari::core::winit::event_loop::ControlFlow;
use hikari::input::*;
use hikari::g3d::*;
use hikari::render::*;
use hikari::pbr::*;
use hikari::core::winit::dpi::*;

pub mod registry;
pub mod settings;

pub use settings::*;

pub struct GameDescription {
    pub name: String,
    pub asset_dir: Option<PathBuf>,
    pub version: String,
    pub worlds: Vec<PathBuf>, //Relative to asset dir
    pub starting_world_ix: Option<usize>,
    // Window settings
    pub initial_window_size: (u32, u32),
    pub resizable: bool,
    pub graphics_settings: GraphicsSettings,
}
impl Default for GameDescription {
    fn default() -> Self {
        Self {
            name: "Untitled".to_string(),
            asset_dir: None,
            version: "0.1.0".to_string(),
            initial_window_size: (1920, 1080),
            starting_world_ix: None,
            worlds: vec![],
            resizable: true,
            graphics_settings: GraphicsSettings::default(),
        }
    }
}

pub struct DefaultRuntime {
    game: Game,
}

impl DefaultRuntime {
    fn load_world(world_path: &Path, asset_manager: &AssetManager) -> anyhow::Result<World> {
        log::debug!("Load world");
        let starting_world = asset_manager.load::<World>(world_path, None, false)?;

        println!("Handle index {:?}", starting_world.index());
        let status = asset_manager.wait_for_load(&starting_world);

        println!("{:?}", status);
        if status != LoadStatus::Loaded {
            return Err(anyhow::anyhow!("Failed to load starting world"));
        }
        let mut world_pool = asset_manager.write_assets::<World>().unwrap();

        assert!(starting_world.strong_count() == 1);
        // assert!(world_pool.get(&starting_world).is_some());
        let new_world = world_pool.take(&starting_world).unwrap();
        println!("Got world");
        Ok(new_world)
    }
    pub fn new(desc: GameDescription) -> anyhow::Result<Self> {   
        let (width, height) = desc.initial_window_size;

        let window = hikari::core::winit::window::WindowBuilder::new()
        .with_title(&desc.name)
        .with_inner_size(LogicalSize::new(width, height))
        .with_resizable(desc.resizable);
        let mut game = Game::new(window)?;
    
        game.add_plugin(CorePlugin);
        game.add_plugin(InputPlugin);
    
        game.add_plugin(GfxPlugin {
            config: GfxConfig {
                debug: false,
                features: Features::default(),
                vsync: desc.graphics_settings.vsync,
            },
        });
    
        game.add_plugin(Plugin3D);

        let pbr_settings = desc.graphics_settings.to_pbr_settings();
        game.add_plugin(PBRPlugin {
            width,
            height,
            settings: pbr_settings
        });
        
        let registry = registry::default_registry();
        game.add_state(registry);
        
        game.add_plugin(WorldLoaderPlugin);

        if let Some(asset_dir) = &desc.asset_dir {
            game.set_asset_dir(asset_dir);
            std::env::set_current_dir(asset_dir)?;
        }

        game.add_state(desc);
        game.add_state(Instant::now());
        game.create_init_stage("Load Dem Assets");
        game.add_init_task("Load Dem Assets", 
        Task::new("Load first world",
        |game_world: &mut World, desc: &GameDescription, asset_manager: &AssetManager| {
            if let Some(starting_world_ix) = desc.starting_world_ix {
                let world_path = &desc.worlds[starting_world_ix];
                *game_world = Self::load_world(world_path, asset_manager)?;
            }

            Ok(())
        }));

        game.add_task(hikari::core::UPDATE, Task::new("Switch world", |game_world: &mut World, now: &Instant, asset_manager: &AssetManager| {
            //let world_path = &desc.worlds[starting_world_ix];
            if now.elapsed() < Duration::new(5, 0) {
                return;
            }
            *game_world = Self::load_world(Path::new("Sponza2.hworld"), asset_manager).expect("Failed to load world");
        }));

        game.add_platform_event_hook(|_, _, event, control_flow| {
            match event {   
                winit::event::Event::WindowEvent { window_id: _, event } => {
                    if matches!(event, winit::event::WindowEvent::CloseRequested) {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                _=> {}
            }
        });
        Ok(
            Self {
                game,
            }
        )
    }
    pub fn with_plugin(mut self, plugin: impl Plugin) -> Self {
        self.game.add_plugin(plugin);

        self
    }
    pub fn run(self) -> anyhow::Result<()> {
        self.game.run();
    }
}