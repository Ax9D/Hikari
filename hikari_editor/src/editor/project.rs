use std::path::Path;

use hikari::{
    asset::{AssetManager, AssetStorage, Handle, LoadStatus},
    core::World,
};
use hikari_editor::{project::Project, Scene, SCENE_EXTENSION};
use std::path::PathBuf;

use crate::{components::EditorComponents, imgui};

use hikari_editor::*;
use hikari_imgui::*;

use super::Editor;

#[derive(Default)]
pub struct ProjectManager {
    pub current: Option<(PathBuf, Project)>,
    current_scene: Option<Handle<Scene>>,
    new_scene_scratch: Option<Handle<Scene>>,
    scene_creator: SceneCreator,
}

#[derive(Default)]
struct SceneCreator {
    name: String,
    path: PathBuf,
}

impl SceneCreator {
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
        let mut scene_file = PathBuf::from(&self.name);
        scene_file.set_extension(SCENE_EXTENSION);
        let potential_filepath = self.path.join(scene_file);
        if potential_filepath.exists() {
            let _color_token = ui.push_style_color(StyleColor::Text, [1.0, 0.0, 0.0, 1.0]);
            ui.text("Scene already exists at that location");
            valid = false;
        }

        let mut new_scene = None;

        ui.disabled(
            !valid || path_string.is_empty() || self.name.is_empty(),
            || {
                if ui.button("Create") {
                    let path = self.path.clone();
                    let name = self.name.clone();

                    let mut full_path = path.join(name);
                    full_path.set_extension(SCENE_EXTENSION);
                    self.path.clear();
                    self.name.clear();

                    ui.close_current_popup();

                    new_scene = Some(full_path);
                }
            },
        );
        ui.same_line();

        if ui.button("Cancel") {
            ui.close_current_popup();
            return None;
        }

        new_scene
    }
}

impl ProjectManager {
    pub fn open(&mut self, file: impl AsRef<Path>, state: EngineState) {
        let file_clone = file.as_ref().to_owned();

        let proj_dir = file_clone.parent().unwrap();
        state.get_mut::<AssetManager>().unwrap().set_asset_dir(proj_dir);

        match Project::open(file) {
            Ok(project) => {
                let mut new_title = String::from("Hikari Editor - ");
                new_title.push_str(&project.name);
                state
                    .get_mut::<&'static winit::window::Window>()
                    .unwrap()
                    .set_title(&new_title);

                
                self.current = Some((file_clone.clone(), project));

                // let folder = file_clone.parent().expect("Failed to find parent folder");
                // std::env::set_current_dir(folder).expect("Failed to set cwd");
            }
            Err(err) => {
                log::error!("Failed to load project: {}", err);
            }
        }
    }
    pub fn set_scene(&mut self, handle: Handle<Scene>, state: EngineState) -> anyhow::Result<()> {
        let components = state.get::<EditorComponents>().unwrap();
        let mut world = state.get_mut::<World>().unwrap();
        let mut storage = state.get_mut::<AssetStorage>().unwrap();
        let scenes = storage.get_mut::<Scene>().unwrap();

        let scene = scenes.get_mut(&handle).unwrap();

        let mut new_world = World::new();

        for entity_ref in scene.world.entities() {
            for component in entity_ref.component_types() {
                if let Some(dispatch) = components.get(component) {
                    dispatch.clone_component(entity_ref.entity(), &scene.world, &mut new_world)?;
                }
            }
        }

        std::mem::swap::<World>(&mut world, &mut new_world);
        self.current_scene = Some(handle.clone());
        

        Ok(())
    }
    pub fn save_all(&self, state: EngineState) -> anyhow::Result<()> {
        let manager = state.get::<AssetManager>().unwrap();
        let mut storage = state.get_mut::<AssetStorage>().unwrap();
        let components = state.get::<EditorComponents>().unwrap();

        let world = state.get::<World>().unwrap();

        if let Some(handle) = &self.current_scene {
            let scenes = storage.get_mut::<Scene>().unwrap();
            let scene = scenes.get_mut(handle).unwrap();
            scene.world.clear();

            for entity_ref in world.entities() {
                for component in entity_ref.component_types() {
                    if let Some(dispatch) = components.get(component) {
                        dispatch.clone_component(entity_ref.entity(), &world, &mut scene.world)?;
                    }
                }
            }

            manager.save(handle, scenes)?;
        }
        if let Some((path, project)) = &self.current {
            project.save(path)?;
        }
        Ok(())
    }
}
pub fn draw(ui: &imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
    let project_manager = &mut editor.project_manager;
    
    let mut new_scene_sure = false;
    {
        let mut storage = state.get_mut::<AssetStorage>().unwrap();
        let manager = state.get::<AssetManager>().unwrap();

        ui.window("Project")
            .size([300.0, 400.0], imgui::Condition::Once)
            .resizable(true)
            .build(|| {
                if let Some((_project_path, project)) = &mut project_manager.current {
                    if ui.button("New Scene") {
                        ui.open_popup("Create Scene");
                    }

                    ui.popup_modal("Create Scene")
                    .collapsible(false)
                    .always_auto_resize(true)
                    .build(ui, || {
                            if let Some(path) = project_manager.scene_creator.draw(ui) {
                                let scene = Scene {
                                    world: World::new(),
                                };
                                project_manager.new_scene_scratch = Some(
                                    project
                                        .create_scene(path, scene, &manager, &mut storage)
                                        .expect("Failed to create project"),
                                );
                            }
                        });

                    for handle in project.scenes() {
                        let scene_path = manager.get_path(handle);
                        let scene_name = scene_path.file_stem().unwrap().to_str().unwrap();

                        if manager.load_status(&handle.clone_erased_as_internal()) != Some(LoadStatus::Loaded) {
                            continue;
                        }
                        let selected = if let Some(current_handle) = &project_manager.current_scene
                        {
                            current_handle == handle
                        } else {
                            false
                        };

                        ui.selectable_config(scene_name).selected(selected).build();

                        if ui.is_double_click(MouseButton::Left) {
                            project_manager.new_scene_scratch = Some(handle.clone());
                            ui.open_popup("Open Scene");
                        }
                    }

                    ui.popup_modal("Open Scene")
                    .collapsible(false)
                    .always_auto_resize(true)
                    .build(ui, || {
                        ui.text("You may have unsaved changes. Are you sure you want to open a new scene?");

                        if ui.button("Yes") {
                            new_scene_sure = true;
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
    if new_scene_sure {
        if let Some(new_scene) = project_manager.new_scene_scratch.take() {
            project_manager.set_scene(new_scene, state)?;
            editor.outliner.reset();
        }
    }
    Ok(())
}
