use std::time::Instant;

use hikari::imgui::Ui;
use hikari_editor::EngineState;

use super::{Editor};

pub enum ImguiSettingsEvent {
    LoadSettings(String),
    SaveSettings,
}

pub struct SaveAndExit {
    pub close_requested: bool,
    pub should_close: bool,
    pub last_autosave_instant: Instant,
    pub imgui_settings_event: Option<ImguiSettingsEvent>,
}
impl Default for SaveAndExit {
    fn default() -> Self {
        Self { 
            close_requested: false,
            should_close: false, 
            last_autosave_instant: Instant::now(),
            imgui_settings_event: None, 
        }
    }
}
impl SaveAndExit {
    fn handle_autosave(editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
        let save_exit = &mut editor.save_and_exit;
        let settings = &mut editor.editor_settings;

        let duration = save_exit.last_autosave_instant.elapsed();
        if settings.autosave_enabled && duration.as_secs() > settings.autosave_interval() {
            editor.save_all(state)?;

            editor.save_and_exit.last_autosave_instant = Instant::now();
        }

        Ok(())
    }
    pub fn draw(ui: &Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
        Self::handle_autosave(editor, state)?;

        let save_exit = &mut editor.save_and_exit;

        if save_exit.close_requested {
            ui.open_popup("Close Editor");
        }

        let mut popup_err = Ok(());

        let mut exit = false;
        let mut save = false;

        ui.modal_popup_config("Close Editor")
        .resizable(false)
        .save_settings(false)
        .collapsible(false)
        .always_auto_resize(true)
        .build(|| {
            ui.text("Are you sure you want to exit? There might be unsaved changes");
            if ui.button("Save And Exit") {
                save = true;
                exit = true;
            };
            ui.same_line();
            
            if ui.button("Dont Save") {
                exit = true;
            }
            ui.same_line();
            
            if ui.button("Cancel") {
                save_exit.close_requested = false;
                ui.close_current_popup();
            }
        });

        if save {
            popup_err = editor.save_all(state);
        } 
        if exit {
            editor.save_and_exit.should_close = true;
            editor.handle_exit();
        }
        popup_err
    }
    pub fn trigger_load_imgui_settings(&mut self, settings: String) {
        self.imgui_settings_event = Some(ImguiSettingsEvent::LoadSettings(settings));
    }
    pub fn trigger_save_imgui_settings(&mut self) {
        self.imgui_settings_event = Some(ImguiSettingsEvent::SaveSettings);
    }
    pub fn load_imgui_settings(editor: &mut Editor, context: &mut hikari::imgui::Context) {
        let save_and_exit = &mut editor.save_and_exit;

        if let Some(ImguiSettingsEvent::LoadSettings(settings)) = &save_and_exit.imgui_settings_event {
            context.load_ini_settings(&settings);

            save_and_exit.imgui_settings_event.take();
        }
    }
    pub fn save_imgui_settings(editor: &mut Editor, context: &mut hikari::imgui::Context) {
        let save_and_exit = &mut editor.save_and_exit;
        let project_manager = &mut editor.project_manager;

        if let Some(ImguiSettingsEvent::SaveSettings) = &save_and_exit.imgui_settings_event {
            let mut buffer = String::new();

            context.save_ini_settings(&mut buffer);
            let project_path = project_manager.current_project_path().unwrap();

            let result = std::fs::write(project_path.join("imgui.ini"), buffer);

            if let Err(err) = result {
                log::error!("Failed to save imgui settings: {}", err);
            }

            save_and_exit.imgui_settings_event.take();
        }
    }
}