use crate::{component_impls, components::EditorComponents, widgets::RenameState};
use clipboard::ClipboardProvider;
use hikari::{
    core::{Game, Registry},
    input::KeyCode,
};
use hikari_editor::*;

mod font;
mod icons;
pub(crate) mod logging;
pub mod meta;
pub mod camera;
mod serialize;
mod style;
mod windows;
mod assets;

use windows::*;

pub use logging::*;
pub use windows::Editor;

struct Clipboard(clipboard::ClipboardContext);
impl Clipboard {
    pub fn new() -> Option<Self> {
        clipboard::ClipboardContext::new()
            .ok()
            .map(|context| Self(context))
    }
}
impl hikari::imgui::ClipboardBackend for Clipboard {
    fn get(&mut self) -> Option<String> {
        self.0.get_contents().ok().map(|text| text.into())
    }

    fn set(&mut self, text: &str) {
        let _ = self.0.set_contents(text.to_owned());
    }
}
pub struct EditorConfig {
    pub log_listener: LogListener,
    pub hidpi_factor: f32,
}

impl Editor {
    pub fn init(game: &mut Game, ctx: &mut hikari::imgui::Context, config: EditorConfig) {
        ctx.io_mut().config_flags = hikari::imgui::ConfigFlags::DOCKING_ENABLE;
        ctx.set_ini_filename(None);
        font::load_fonts(ctx, &config);
        style::set_dark_theme(ctx);

        if let Some(clipboard) = Clipboard::new() {
            ctx.set_clipboard_backend(clipboard);
        } else {
            log::error!("Failed to init clipboard");
        }

        let mut editor_components = EditorComponents::default();
        let mut registry = Registry::builder();

        component_impls::register_components(&mut editor_components, &mut registry);

        let registry = registry.build();
        game.add_state(registry);
        game.add_plugin(hikari::core::load_save::WorldLoaderPlugin);
        //game.create_asset::<Scene>();
        // let loader = SceneLoader { registry };
        // game.register_asset_loader::<Scene, SceneLoader>(loader.clone());
        // game.register_asset_saver::<Scene, SceneLoader>(loader);


        let editor = Editor {
            logging: Logging::new(config.log_listener),
            debugger: Debugger::new(),
            show_demo: false,
            content_browser: ContentBrowser::new(),
            outliner: Outliner::default(),
            properties: Properties::default(),
            viewport: Viewport::default(),
            rename_state: RenameState::Idle,
            project_manager: ProjectManager::default(),
            material_editor: MaterialEditor::default(),
            about: About::default(),
            save_and_exit: SaveAndExit::default(),
            editor_settings: EditorSettings::default(),
            render_settings: RenderSettings::default(),
        };

        game.add_state(editor);
        game.add_state(editor_components);
    }
    pub fn update(&mut self, ui: &hikari::imgui::Ui, state: EngineState) {
        use hikari::imgui;
        ui.window("Main")
            .flags(
                imgui::WindowFlags::NO_DECORATION
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::MENU_BAR
                    | imgui::WindowFlags::NO_DOCKING
                    | imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS
                    | imgui::WindowFlags::NO_NAV_FOCUS,
            )
            .position([0.0, 0.0], imgui::Condition::Always)
            .size(ui.io().display_size, imgui::Condition::Always)
            .build(|| {
                ui.menu_bar(|| {
                    //project::draw(ui, self, state).unwrap();
                    self.file_menu(ui, state).unwrap();

                    ui.menu("Edit", || {
                        ui.menu_item_config("Preferences").enabled(false).build();
                    });
                    ui.menu("Windows", || {
                        if ui.menu_item("Material Editor") {
                            MaterialEditor::open(self);
                        }
                    });
                    ui.menu("Settings", || {
                        if ui.menu_item("Render Settings") {
                            RenderSettings::open(self);
                        }
                        if ui.menu_item("Editor Settings") {
                            EditorSettings::open(self);
                        }
                    });
                    ui.menu("Tools", || {
                        if ui.menu_item("Debugger") {
                            Debugger::open(self);
                        }
                        if ui.menu_item("Start Tracy") {
                            let path = hikari::utils::engine_dir().join("data/tools/");

                            #[cfg(target_os = "windows")]
                            let tracy_exe = "Tracy.exe";
                            #[cfg(target_os = "linux")]
                            let tracy_exe = "tracy";

                            let result = std::process::Command::new(path.join(tracy_exe)).spawn();

                            if let Err(err) = result {
                                log::error!("Failed to spawn tracy: {}", err);
                            }
                        }
                    });
                    ui.menu("Help", || {
                        self.show_demo = ui.menu_item_config("Demo Window").build();
                        if ui.menu_item("Copy Style to Clipboard as Rust") {
                            style::copy_style_to_clipboard_as_rust(ui);
                        }
                        if ui.menu_item("About") {
                            self.about.open();
                        }
                    });
                });
                self.default_layout(ui);
            });

        let result = self.draw_windows(ui, state);
        if let Err(err) = result {
            log::error!("{}", err);
        }
    }
    pub fn file_menu(&mut self, ui: &hikari::imgui::Ui, state: EngineState) -> anyhow::Result<()> {
        let mut open = false;
        let mut save = false;
        let project_open = self.project_manager.is_project_open();

        ui.menu("File", || {
            open |= ui.menu_item_config("Open").shortcut("Ctrl + O").build();

            ui.text("Auto Save");
            ui.same_line();
            {
            let _width_token = ui.push_item_width(-1.0);
            ui.checkbox("###Auto Save", &mut self.editor_settings.autosave_enabled);
            }
            save |= ui
                .menu_item_config("Save All")
                .shortcut("Ctrl + S")
                .enabled(project_open)
                .build();
        });

        let input = state.get::<hikari::input::Input>().unwrap();
        let keyboard = input.keyboard();
        open |= ui.io().key_ctrl && keyboard.was_just_pressed(KeyCode::O); // Ctrl + O

        save |= project_open && ui.io().key_ctrl && keyboard.was_just_pressed(KeyCode::S); // Ctrl + S

        if open {
            if let Some(project_file) = rfd::FileDialog::new()
                .add_filter("Hikari Project", &["hikari"])
                .pick_file()
            {
                self.load(&project_file, state)?;
            }
        }

        if save {
            self.save_all(state)?;
        }

        Ok(())
    }
    pub fn draw_windows(
        &mut self,
        ui: &hikari::imgui::Ui,
        state: EngineState,
    ) -> anyhow::Result<()> {
        hikari::dev::profile_function!();
        //Update render settings before render, so incase of a resize we don't use freed resources in the imgui pass
        RenderSettings::draw_if_open(ui, self, state)?;
        EditorSettings::draw_if_open(ui, self, state)?;

        //content_browser::draw(ui, self, state).unwrap();
        Viewport::draw_if_open(ui, self, state)?;
        Outliner::draw_if_open(ui, self, state)?;
        ProjectManager::draw_if_open(ui, self, state)?;
        Properties::draw_if_open(ui, self, state)?;
        Logging::draw_if_open(ui, self, state)?;
        Debugger::draw_if_open(ui, self, state)?;
        About::draw_if_open(ui, self, state)?;

        MaterialEditor::draw_if_open(ui, self, state)?;


        SaveAndExit::draw(ui, self, state)?;

        if self.show_demo {
            ui.show_demo_window(&mut self.show_demo);
        }

        Ok(())
    }
    pub fn load(&mut self, project_file: &std::path::Path, state: EngineState) -> anyhow::Result<()> {
        self.project_manager.open(project_file, state)?;
        self.load_state()
    }
    pub fn save_all(&mut self, state: EngineState) -> anyhow::Result<()> {
        self.project_manager.save_all(state)?;
        self.save_state()
    }
    fn load_state(&mut self) -> anyhow::Result<()> {
        if let Some(project_path) = self.project_manager.current_project_path() {
            let project_path = project_path.to_path_buf();
            let path = project_path.join("editor.yaml");

            if !path.exists() {
                self.save_state()?;
                return Ok(());
            }

            let file = std::fs::OpenOptions::new().read(true).open(path)?;
            let deserializer = serde_yaml::Deserializer::from_reader(file);

            self.deserialize(deserializer)?;

            if let Some(settings) = std::fs::read_to_string(project_path.join("imgui.ini")).ok() {
                self.save_and_exit.trigger_load_imgui_settings(settings);
            }
        }
        Ok(())
    }
    fn save_state(&mut self) -> anyhow::Result<()> {
        if let Some(project_path) = self.project_manager.current_project_path() {
            let path = project_path.join("editor.yaml");
            let file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?;
            let mut serializer = serde_yaml::Serializer::new(file);
            self.serialize(&mut serializer)?;
            
            self.save_and_exit.trigger_save_imgui_settings();
        }
        
        Ok(())
    }
    pub fn pre_update(
        &mut self,
        _window: &winit::window::Window,
        _context: &mut hikari::imgui::Context,
    ) {
    }
    pub fn post_update(
        &mut self,
        _window: &winit::window::Window,
        context: &mut hikari::imgui::Context,
    ) {
        SaveAndExit::load_imgui_settings(self, context);
        SaveAndExit::save_imgui_settings(self, context);
    }
    pub fn should_close(&self) -> bool {
        self.save_and_exit.should_close
    }
    pub fn request_close(&mut self) {
        self.save_and_exit.close_requested = true;
    }
    pub fn handle_exit(&mut self) {
        log::info!("Editor Exiting");
    }
}
