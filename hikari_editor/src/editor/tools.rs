use super::Editor;
use crate::imgui;
use hikari_editor::*;

pub struct Tools {}
impl Tools {
    pub fn new() -> Self {
        Self {}
    }
}
pub fn draw(ui: &imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
    Ok(())
}
