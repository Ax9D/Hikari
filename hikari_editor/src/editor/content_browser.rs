use std::path::{Path, PathBuf};

use crate::imgui;

use super::{Editor};

pub struct ContentBrowser {
    cwd: PathBuf,
}
impl ContentBrowser {
    pub fn new() -> Self {
        Self { cwd: ".".into() }
    }
}

pub fn draw(ui: &imgui::Ui, editor: &mut Editor) {
    ui.window("Content Browser")
        .size([950.0, 200.0], imgui::Condition::Once)
        .resizable(true)
        .build(|| {
            let now = std::time::Instant::now();
            if ui.button("Back") && editor.content_browser.cwd != Path::new(".") {
                editor.content_browser.cwd.pop();
            }
            ui.separator();
            for entry in std::fs::read_dir(&editor.content_browser.cwd).unwrap() {
                let entry = entry.unwrap();
                if entry.path().is_dir() {
                    ui.button(entry.file_name().to_str().unwrap());
                    let folder_dbl_click = ui.is_mouse_clicked(imgui::MouseButton::Left);
                    if folder_dbl_click {
                        editor.content_browser.cwd.push(entry.file_name())
                    }
                } else {
                    ui.text(entry.file_name().to_str().unwrap());
                }
            }
            ui.text(format!("Time taken {:?}", now.elapsed()));
        });
}
