use hikari::imgui::{Condition, Ui};
use serde::{Serialize, Deserialize};

use super::EditorWindow;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct EditorSettings {
    pub autosave_enabled: bool,
    autosave_interval: u64,
    open: bool
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self { 
            autosave_enabled: false, 
            autosave_interval: 10,
            open: false
        }
    }
}

impl EditorSettings {
    pub fn autosave_interval(&self) -> u64 {
        self.autosave_interval
    }
}

impl EditorWindow for EditorSettings {
    fn open(editor: &mut super::Editor) {
        editor.editor_settings.open = true;
    }
    fn is_open(editor: &mut super::Editor) -> bool {
        editor.editor_settings.open
    }
    fn draw(ui: &Ui, editor: &mut super::Editor, _state: hikari_editor::EngineState) -> anyhow::Result<()> {
        let settings = &mut editor.editor_settings;

        ui.window("Editor Settings")
        .opened(&mut settings.open)
        .size([400.0, 400.0], Condition::FirstUseEver)
        .resizable(true)
        .build(|| {
            ui.checkbox("Auto Save", &mut settings.autosave_enabled);

            ui.disabled(!settings.autosave_enabled, || {
                let mut new_interval = settings.autosave_interval;
                
                ui.input_scalar("Auto Save Interval (seconds)", &mut new_interval)
                .build();

                let changed = ui.is_item_deactivated_after_edit();

                new_interval = new_interval.clamp(2, u64::MAX);

                ui.same_line();

                if changed {
                    settings.autosave_interval = new_interval;
                }
            });
        });
        
        Ok(())
    }
}