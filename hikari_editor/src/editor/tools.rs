use crate::{imgui, EngineState};

use super::Editor;

pub struct Tools {}
impl Tools {
    pub fn new() -> Self {
        Self {}
    }
}
pub fn draw(ui: &imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
    Ok(())
}
