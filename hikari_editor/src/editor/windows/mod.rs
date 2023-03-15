mod about;
mod asset_editors;
mod camera;
mod content_browser;
mod debugger;
mod logging;
mod outliner;
mod project;
mod properties;
mod render_settings;
mod viewport;

pub use about::*;
pub use asset_editors::*;
pub use camera::*;
pub use content_browser::*;
pub use debugger::*;
use hikari_editor::EngineState;
pub use logging::*;
pub use outliner::*;
pub use project::*;
pub use properties::*;
pub use render_settings::*;
pub use viewport::*;

use crate::widgets::RenameState;

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
    pub fn default_layout(&self, ui: &hikari::imgui::Ui) {
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
}
