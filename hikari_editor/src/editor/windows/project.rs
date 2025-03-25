use std::path::Path;

use hikari::{
    asset::{AssetManager, Handle, LoadStatus},
    core::{Registry, World},
    g3d::Camera
};
use hikari_editor::{project::Project};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

use crate::{editor::{meta::EditorOnly, assets}, imgui};

use hikari::imgui::*;
use hikari_editor::*;

use super::{Editor, EditorWindow};

#[derive(Default)]
struct WorldCreator {
    name: String,
    path: PathBuf,
}

impl WorldCreator {
    pub fn draw(&mut self, ui: &Ui) -> Option<PathBuf> {
        ui.input_text("Name", &mut self.name).build();
        let mut path_string = self.path.display().to_string();

        ui.input_text("Path", &mut path_string)
            .read_only(true)
            .build();

        ui.same_line();

        if ui.button("Browse") {
            if let Some(folder) = rfd::FileDialog::new()
                .set_directory(std::env::current_dir().unwrap())
                .pick_folder()
            {
                self.path = folder;
            }
        }

        let mut valid = true;
        let mut world_file = PathBuf::from(&self.name);
        world_file.set_extension(hikari::core::load_save::SUPPORTED_WORLD_EXTENSIONS[0]);
        let potential_filepath = self.path.join(world_file);
        if potential_filepath.exists() {
            let _color_token = ui.push_style_color(StyleColor::Text, [1.0, 0.0, 0.0, 1.0]);
            ui.text("World already exists at that location");
            valid = false;
        }

        let mut new_world = None;

        ui.disabled(
            !valid || path_string.is_empty() || self.name.is_empty(),
            || {
                if ui.button("Create") {
                    let path = self
                        .path
                        .strip_prefix(std::env::current_dir().unwrap())
                        .unwrap();
                    let name = self.name.clone();

                    let mut full_path = path.join(name);
                    full_path.set_extension(hikari::core::load_save::SUPPORTED_WORLD_EXTENSIONS[0]);
                    self.path.clear();
                    self.name.clear();

                    ui.close_current_popup();

                    new_world = Some(full_path);
                }
            },
        );
        ui.same_line();

        if ui.button("Cancel") {
            ui.close_current_popup();
            return None;
        }

        new_world
    }
}
#[derive(Default)]
pub struct ProjectManager {
    current: Option<(PathBuf, Project)>,
    current_world: Option<Handle<World>>,
    current_world_ix: Option<usize>,
    new_world_scratch: Option<usize>,
    world_creator: WorldCreator,
}

impl ProjectManager {
    pub fn open(&mut self, file: impl AsRef<Path>, state: EngineState) -> anyhow::Result<()> {
        let file_clone = file.as_ref();

        let proj_dir = file_clone.parent().unwrap();

        let asset_manager = state.get::<AssetManager>().unwrap();

        asset_manager.set_asset_dir(proj_dir)?;
        std::env::set_current_dir(proj_dir)?;

        match Project::open(&file) {
            Ok(project) => {
                let mut new_title = String::from("Hikari Editor - ");
                new_title.push_str(&project.name);
                state
                    .get::<winit::window::Window>()
                    .unwrap()
                    .set_title(&new_title);

                self.current = Some((proj_dir.to_owned(), project));

                self.load_render_settings(state, proj_dir)?;
            }
            Err(err) => {
                log::error!("Failed to load project: {}", err);
            }
        }

        Ok(())
    }
    pub fn set_world(&mut self, world_ix: usize, state: EngineState) -> anyhow::Result<()> {
        let registry = state.get::<Registry>().unwrap();
        let mut game_world = state.get_mut::<World>().unwrap();
        let manager = state.get::<AssetManager>().unwrap();

        let (_, project) = self.current.as_ref().ok_or(anyhow::anyhow!("No project open!"))?;
        let world_path = &project.worlds()[world_ix];

        let handle = manager.load(world_path, None, false)?;
        assert!(!handle.is_weak());
        let status = manager.wait_for_load(&handle);

        
        if status != LoadStatus::Loaded {
            return Err(anyhow::anyhow!("Failed to load World"));
        }
        log::info!("Loaded world {:?}", world_path);
        
        let mut worlds = manager.write_assets::<World>().unwrap();

        let world = worlds.get_mut(&handle).unwrap();

        let new_world = world.clone(&registry);

        *game_world = new_world;

        if let Some(old_world_handle) = self.current_world.replace(handle) {
            manager.mark_unsaved(&old_world_handle);
        }

        self.current_world_ix.replace(world_ix);
        Ok(())
    }
    fn load_render_settings(&self, state: EngineState, path: &Path) -> anyhow::Result<()> {
        let settings_name = Path::new("render_settings.yaml");
        let settings_path = path.join(settings_name);

        if !settings_path.exists() {
            return self.save_render_settings(state, path);
        }

        let file = std::fs::OpenOptions::new().read(true).open(settings_path)?;
        let deserializer = serde_yaml::Deserializer::from_reader(file);
        let new_settings = hikari::pbr::Settings::deserialize(deserializer)?;

        let mut gfx = state.get_mut::<hikari::render::Gfx>().unwrap();
        let mut renderer = state.get_mut::<hikari::pbr::WorldRenderer>().unwrap();
        let mut shader_library = state.get_mut::<hikari::g3d::ShaderLibrary>().unwrap();

        renderer.update_settings(&mut gfx, &mut shader_library, 
            |settings| {
            *settings = new_settings;
        })?;
        
        Ok(())
    }
    fn save_render_settings(&self, state: EngineState, path: &Path) -> anyhow::Result<()> {
        let renderer = state.get::<hikari::pbr::WorldRenderer>().unwrap();

        let settings = renderer.settings();
        
        let settings_name = Path::new("render_settings.yaml");
        let settings_path = path.join(settings_name);

        let file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(settings_path)?;
        let mut serializer = serde_yaml::Serializer::new(file);

        settings.serialize(&mut serializer)?;

        Ok(())
    }
    pub fn save_all(&mut self, state: EngineState) -> anyhow::Result<()> {
        let manager = state.get::<AssetManager>().unwrap();
        let registry = state.get::<Registry>().unwrap();

        let game_world = state.get::<World>().unwrap();
        
        if let Some((path, project)) = &self.current {
            if let Some(handle) = &self.current_world {
                let mut worlds = manager.write_assets::<World>().unwrap();
                let world = worlds.get_mut(handle).unwrap();
                world.clear();

                game_world.clone_into(&registry, world);

                manager.mark_unsaved(handle);
            }
            
            assets::save_all(&manager, true)?;

            project.save(path)?;
            manager.save_db()?;
            self.save_render_settings(state, path)?;
            log::info!("Saved Project");
        }
        Ok(())
    }
    pub fn is_project_open(&self) -> bool {
        self.current.is_some()
    }
    pub fn current_project_path(&self) -> Option<&Path> {
        self.current.as_ref().map(|(path, _)| path.as_path())
    }
    pub fn current_world(&self) -> Option<&Handle<World>> {
        self.current_world.as_ref()
    }
    pub fn current_world_ix(&self) -> Option<usize> {
        self.current_world_ix
    }
}

fn new_world() -> World {
    let mut world = World::new();
    world.create_entity_with((EditorOnly, Camera::default()));

    world
}

impl EditorWindow for ProjectManager {
    fn draw(ui: &imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
        let project_manager = &mut editor.project_manager;

        let mut new_world_sure = false;
        {
            let manager = state.get::<AssetManager>().unwrap();

            ui.window("Project")
                .size([300.0, 400.0], imgui::Condition::FirstUseEver)
                .resizable(true)
                .build(|| {
                    let current_world_ix = project_manager.current_world_ix();
                    if let Some((_project_path, project)) = &mut project_manager.current {
                        if ui.button("New World") {
                            ui.open_popup("Create World");
                        }

                        ui.modal_popup_config("Create World")
                        .collapsible(false)
                        .always_auto_resize(true)
                        .build(|| {
                                if let Some(path) = project_manager.world_creator.draw(ui) {
                                    let (_, world_ix) = project
                                    .add_world(path, new_world(), &manager)
                                    .expect("Failed to create world");
                                    project_manager.new_world_scratch = Some(world_ix);
                                }
                            });

                        for (ix, world_path) in project.worlds().iter().enumerate() {
                            let world_name = world_path.file_stem().unwrap().to_str().unwrap();

                            let selected = if let Some(current_ix) = current_world_ix
                            {
                                current_ix == ix
                            } else {
                                false
                            };

                            ui.selectable_config(world_name)
                            .selected(selected)
                            
                            .build();

                            if ui.is_double_click(MouseButton::Left) {
                                project_manager.new_world_scratch = Some(ix);
                                ui.open_popup("Open World");
                            }
                        }

                        ui.modal_popup_config("Open World")
                        .resizable(false)
                        .save_settings(false)
                        .collapsible(false)
                        .always_auto_resize(true)
                        .build(|| {
                            ui.text("You may have unsaved changes. Are you sure you want to open a new world?");

                            if ui.button("Yes") {
                                new_world_sure = true;
                                ui.close_current_popup();
                            }
                            ui.same_line();
                            if ui.button("No") {
                                ui.close_current_popup();
                            }
                        });
                    } else {
                        ui.text("No Project Open");
                    }
                });
        }
        if new_world_sure {
            if let Some(new_world) = project_manager.new_world_scratch.take() {
                project_manager.set_world(new_world, state)?;
                let mut world = state.get_mut::<World>().unwrap();
                editor.outliner.on_world_loaded(&mut world);
            }
        }
        Ok(())
    }
}
