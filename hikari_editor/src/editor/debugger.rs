use super::Editor;
use crate::imgui;
use hikari::{pbr::WorldRenderer, render::imgui_support::TextureExt};
use hikari_editor::*;

pub struct Debugger {
    is_open: bool
}
impl Debugger {
    pub fn new() -> Self {
        Self {
            is_open: false
        }
    }
    pub fn open(&mut self) {
        self.is_open = true;
    }
}
pub fn draw(ui: &imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
    if !editor.debugger.is_open {
        return Ok(());
    }

    ui.window("Debugger")        
    .size([400.0, 400.0], imgui::Condition::Once)
    .resizable(true)
    .opened(&mut editor.debugger.is_open)
    .build(|| {
        if let Some(_token) = ui.tab_bar("Render Graph Tabs") {
            let renderer = state.get::<WorldRenderer>().unwrap();

            if let Some(_token) = ui.tab_item("Images") {
                let resources = renderer.graph_resources();
                let images = resources.image_handles();
                for (name, handle) in images {
                    let image = resources.get_image(handle).unwrap();
                    if let Some(_token) = ui.tree_node(&name) {
                        ui.text(format!("VkImage: {:?}", image.image()));
                        ui.text(format!("Config: {:#?}", image.config()));
                    }
                }
            }
            // if let Some(_token) = ui.tab_item("Render Target Debug") {
            //     ui.text("Shadow Map Atlas");
            //     let shadow_map = renderer.graph_resources().get_image_by_name("ShadowMapAtlasDebug").unwrap();

            //     imgui::Image::new(ui.get_texture_id(shadow_map), [400.0 * 4.0, 400.0]).build(ui);

            //     ui.text("Z Prepass");
            //     let depth_map = renderer.graph_resources().get_image_by_name("PrepassDepthDebug").unwrap();

            //     imgui::Image::new(ui.get_texture_id(depth_map), [400.0, 400.0]).build(ui);
            // }
        }
    });
    Ok(())
}
