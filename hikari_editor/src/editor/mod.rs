use self::{
    about::About,
    asset_editors::MaterialEditor,
    content_browser::ContentBrowser,
    debugger::Debugger,
    logging::{LogListener, Logging},
    outliner::Outliner,
    project::ProjectManager,
    properties::Properties,
    render_settings::RenderSettings,
    viewport::Viewport,
};
use crate::{component_impls, components::EditorComponents};
use clipboard::ClipboardProvider;
use hikari::{
    core::{serde::Registry, Game},
    input::KeyCode,
};
use hikari_editor::*;

pub mod logging;

//mod utils;
mod about;
mod camera;
mod content_browser;
mod debugger;
mod font;
mod icons;
pub mod meta;
mod outliner;
mod project;
mod properties;
mod render_settings;
mod style;
mod viewport;

mod asset_editors;

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

#[derive(PartialEq, Eq)]
pub enum RenameState {
    Idle,
    Renaming(hikari::imgui::Id, String, i32),
}

pub trait EditorWindow {
    fn draw(ui: &hikari::imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()>;
    fn draw_if_open(
        ui: &hikari::imgui::Ui,
        editor: &mut Editor,
        state: EngineState,
    ) -> anyhow::Result<()> {
        if Self::is_open(editor) {
            Self::draw(ui, editor, state)?;
        }
        Ok(())
    }
    fn open(_editor: &mut Editor) {}
    fn is_open(_editor: &mut Editor) -> bool {
        true
    }
}
pub struct Editor {
    pub outliner: Outliner,
    pub properties: Properties,
    pub viewport: Viewport,
    pub content_browser: ContentBrowser,
    pub logging: Logging,
    pub debugger: Debugger,
    pub rename_state: RenameState,
    pub project_manager: ProjectManager,
    pub about: About,
    pub material_editor: MaterialEditor,
    pub show_demo: bool,
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
        let mut registry = Registry::new();

        component_impls::register_components(&mut editor_components, &mut registry);

        let registry = std::sync::Arc::new(registry);

        game.create_asset::<Scene>();
        let loader = SceneLoader { registry };
        game.register_asset_loader::<Scene, SceneLoader>(loader.clone());
        game.register_asset_saver::<Scene, SceneLoader>(loader);

        let editor = Self {
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

        self.draw_windows(ui, state).unwrap();
    }
    fn default_layout(&self, ui: &hikari::imgui::Ui) {
        use hikari::imgui::*;
        if ui.get_node("Dockspace").is_some() {
            let _root = ui.dockspace("Dockspace", [0.0, 0.0], 0);
            return;
        }

        let root = ui.dockspace(
            "Dockspace",
            [0.0, 0.0],
            hikari::imgui::sys::ImGuiDockNodeFlags_AutoHideTabBar as i32,
        );

        root.split(
            hikari::imgui::Direction::Left,
            0.8,
            |left| {
                left.split(
                    hikari::imgui::Direction::Up,
                    0.7,
                    |up| {
                        up.dock_window(ui, "Viewport");
                    },
                    |down| {
                        down.dock_window(ui, "Engine Log");
                    },
                )
            },
            |right| {
                right.split(
                    hikari::imgui::Direction::Up,
                    0.6,
                    |up| {
                        up.dock_window(ui, "Project");
                        up.dock_window(ui, "Outliner");
                        up.dock_window(ui, "Render Settings");
                    },
                    |down| {
                        down.dock_window(ui, "Properties");
                    },
                );
            },
        );
    }
    pub fn file_menu(&mut self, ui: &hikari::imgui::Ui, state: EngineState) -> anyhow::Result<()> {
        let mut open = false;
        let mut save = false;
        let project_open = self.project_manager.current.is_some();

        ui.menu("File", || {
            open |= ui.menu_item_config("Open").shortcut("Ctrl + O").build();

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
                self.project_manager.open(project_file, state)?;
            }
        }

        if save {
            self.project_manager.save_all(state)?;
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

        //content_browser::draw(ui, self, state).unwrap();
        Viewport::draw_if_open(ui, self, state)?;
        Outliner::draw_if_open(ui, self, state)?;
        ProjectManager::draw_if_open(ui, self, state)?;
        Properties::draw_if_open(ui, self, state)?;
        Logging::draw_if_open(ui, self, state)?;
        Debugger::draw_if_open(ui, self, state)?;
        About::draw_if_open(ui, self, state)?;

        MaterialEditor::draw_if_open(ui, self, state)?;

        if self.show_demo {
            ui.show_demo_window(&mut self.show_demo);
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
        self.project_manager.load_imgui_settings(context);
        self.project_manager.save_imgui_settings(context);
    }
    pub fn handle_exit(&mut self) {
        log::info!("Editor Exiting");
    }
}
