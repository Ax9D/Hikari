use hikari_editor::EngineState;

use crate::imgui;
use crate::Editor;

use super::EditorWindow;
#[derive(Default)]
pub struct About {
    is_open: bool,
}

impl About {
    pub fn open(&mut self) {
        self.is_open = true;
    }
}
impl EditorWindow for About {
    fn is_open(editor: &mut Editor) -> bool {
        editor.about.is_open
    }
    fn open(editor: &mut Editor) {
        editor.about.is_open = true;
    }
    fn draw(ui: &imgui::Ui, editor: &mut Editor, _state: EngineState) -> anyhow::Result<()> {
        let about = &mut editor.about;

        ui.window("About")
            .size([500.0, 150.0], imgui::Condition::Always)
            .position_pivot([0.5, 0.5])
            .resizable(false)
            .opened(&mut about.is_open)
            .build(|| {
                let name = "Hikari Editor";
                ui.text(name);
                ui.text(format!("Version: {}", env!("CARGO_PKG_VERSION")));
                ui.text(format!("Commit: {}", env!("GIT_HASH")));
            });

        Ok(())
    }
}
