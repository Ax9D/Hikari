use std::path::{Path, PathBuf};

use super::Editor;
use super::EditorWindow;
use hikari::imgui::*;
use hikari_editor::EngineState;

pub struct ContentBrowser {
    cwd: PathBuf,
}
impl ContentBrowser {
    pub fn new() -> Self {
        Self { cwd: ".".into() }
    }
}
impl EditorWindow for ContentBrowser {
    fn draw(ui: &Ui, editor: &mut Editor, _state: EngineState) -> anyhow::Result<()> {
        ui.window("Content Browser")
            .size([950.0, 200.0], Condition::Once)
            .resizable(true)
            .build(|| {
                if ui.button("Back") && editor.content_browser.cwd != Path::new(".") {
                    editor.content_browser.cwd.pop();
                }
                ui.separator();
                for entry in std::fs::read_dir(&editor.content_browser.cwd).unwrap() {
                    let entry = entry.unwrap();
                    if entry.path().is_dir() {
                        ui.button(entry.file_name().to_str().unwrap());
                        let folder_dbl_click = ui.is_mouse_clicked(MouseButton::Left);
                        if folder_dbl_click {
                            editor.content_browser.cwd.push(entry.file_name())
                        }
                    } else {
                        ui.text(entry.file_name().to_str().unwrap());
                    }
                }
            });

        Ok(())
    }
}
