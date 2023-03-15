use std::path::Path;

use hikari::{
    asset::{AssetManager, Handle, LoadStatus},
    core::{Registry, World},
    g3d::Camera,
};
use hikari_editor::{project::Project, Scene, SCENE_EXTENSION};
use std::path::PathBuf;

use crate::{editor::meta::EditorOnly, imgui};

use hikari::imgui::*;
use hikari_editor::*;

use super::{Editor, EditorWindow};

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
                    let path = self
                        .path
                        .strip_prefix(std::env::current_dir().unwrap())
                        .unwrap();
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
pub enum ImguiSettingsEvent {
    LoadSettings(String),
    SaveSettings,
}
#[derive(Default)]
pub struct ProjectManager {
    current: Option<(PathBuf, Project)>,
    current_scene: Option<Handle<Scene>>,
    new_scene_scratch: Option<Handle<Scene>>,
    scene_creator: SceneCreator,
    imgui_settings_event: Option<ImguiSettingsEvent>,
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
                    .get_mut::<&'static winit::window::Window>()
                    .unwrap()
                    .set_title(&new_title);

                self.current = Some((proj_dir.to_owned(), project));

                if let Some(settings) = std::fs::read_to_string(proj_dir.join("imgui.ini")).ok() {
                    self.imgui_settings_event = Some(ImguiSettingsEvent::LoadSettings(settings));
                }
                // let folder = file_clone.parent().expect("Failed to find parent folder");
                // std::env::set_current_dir(folder).expect("Failed to set cwd");
            }
            Err(err) => {
                log::error!("Failed to load project: {}", err);
            }
        }

        Ok(())
    }
    pub fn set_scene(&mut self, handle: Handle<Scene>, state: EngineState) -> anyhow::Result<()> {
        let registry = state.get::<Registry>().unwrap();
        let mut world = state.get_mut::<World>().unwrap();
        let manager = state.get::<AssetManager>().unwrap();

        manager.request_load(&handle, None, false)?;
        manager.wait_for_load(&handle);

        let mut scenes = manager.write_assets::<Scene>().unwrap();

        let scene = scenes.get_mut(&handle).unwrap();

        let mut new_world = World::new();

        // for entity_ref in scene.world.entities() {
        //     for component in entity_ref.component_types() {
        //         if let Some(dispatch) = components.get(component) {
        //             dispatch.clone_component(entity_ref.entity(), &scene.world, &mut new_world)?;
        //         }
        //     }
        // }

        scene.world.clone_into(&registry, &mut new_world);

        std::mem::swap::<World>(&mut world, &mut new_world);
        self.current_scene = Some(handle.clone());

        Ok(())
    }
    pub fn save_all(&mut self, state: EngineState) -> anyhow::Result<()> {
        let manager = state.get::<AssetManager>().unwrap();
        let registry = state.get::<Registry>().unwrap();

        let world = state.get::<World>().unwrap();

        if let Some(handle) = &self.current_scene {
            let mut scenes = manager.write_assets::<Scene>().unwrap();
            let scene = scenes.get_mut(handle).unwrap();
            scene.world.clear();

            world.clone_into(&registry, &mut scene.world);
            // for entity_ref in world.entities() {
            //     for component in entity_ref.component_types() {
            //         if let Some(dispatch) = components.get(component) {
            //             dispatch.clone_component(entity_ref.entity(), &world, &mut scene.world)?;
            //         }
            //     }
            // }
            drop(scenes);
            manager.save(handle)?;
        }
        if let Some((path, project)) = &self.current {
            project.save(path)?;
            manager.save_db()?;
            self.imgui_settings_event = Some(ImguiSettingsEvent::SaveSettings);
            log::info!("Saved Project");
        }
        Ok(())
    }
    pub fn load_imgui_settings(&mut self, context: &mut imgui::Context) {
        if let Some(ImguiSettingsEvent::LoadSettings(settings)) = &self.imgui_settings_event {
            context.load_ini_settings(&settings);

            self.imgui_settings_event.take();
        }
    }
    pub fn save_imgui_settings(&mut self, context: &mut imgui::Context) {
        if let Some(ImguiSettingsEvent::SaveSettings) = &self.imgui_settings_event {
            let mut buffer = String::new();

            context.save_ini_settings(&mut buffer);
            let (project_path, _) = self.current.as_ref().unwrap();

            let result = std::fs::write(project_path.join("imgui.ini"), buffer);

            if let Err(err) = result {
                log::error!("Failed to save imgui settings: {}", err);
            }

            self.imgui_settings_event.take();
        }
    }
    pub fn is_project_open(&self) -> bool {
        self.current.is_some()
    }
    pub fn current_project_path(&self) -> Option<&Path> {
        self.current.as_ref().map(|(path, _)| path.as_path())
    }
    pub fn current_scene(&self) -> Option<&Handle<Scene>> {
        self.current_scene.as_ref()
    }
}

fn new_scene() -> Scene {
    let mut world = World::new();
    world.create_entity_with((EditorOnly, Camera::default()));

    Scene { world }
}

impl EditorWindow for ProjectManager {
    fn draw(ui: &imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
        let project_manager = &mut editor.project_manager;

        let mut new_scene_sure = false;
        {
            let manager = state.get::<AssetManager>().unwrap();

            ui.window("Project")
                .size([300.0, 400.0], imgui::Condition::FirstUseEver)
                .resizable(true)
                .build(|| {
                    if let Some((_project_path, project)) = &mut project_manager.current {
                        if ui.button("New Scene") {
                            ui.open_popup("Create Scene");
                        }

                        ui.modal_popup_config("Create Scene")
                        .collapsible(false)
                        .always_auto_resize(true)
                        .build(|| {
                                if let Some(path) = project_manager.scene_creator.draw(ui) {
                                    let scene = new_scene();
                                    project_manager.new_scene_scratch = Some(
                                        project
                                            .create_scene(path, scene, &manager)
                                            .expect("Failed to create scene"),
                                    );
                                }
                            });

                        for handle in project.scenes() {
                            let erased_handle = handle.clone_erased_as_weak();
                            let db = manager.asset_db().read();
                            let scene_path = db.handle_to_path(&erased_handle).unwrap();
                            let scene_name = scene_path.file_stem().unwrap().to_str().unwrap();

                            if manager.status(&erased_handle) == Some(LoadStatus::Failed) {
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

                        ui.modal_popup_config("Open Scene")
                        .resizable(false)
                        .save_settings(false)
                        .collapsible(false)
                        .always_auto_resize(true)
                        .build(|| {
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
}
