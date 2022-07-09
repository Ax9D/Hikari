use hikari::{
    asset::AssetStorage, core::World, pbr::WorldRenderer, render::imgui_support::TextureExt,
};

use crate::{imgui, EngineState};

use super::Editor;

#[derive(Default)]
pub struct Viewport {
    size: (u32, u32),
}
pub fn draw(ui: &imgui::Ui, _editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
    let renderer = state.get_mut::<WorldRenderer>().unwrap();

    let pbr_output = renderer.get_output_image();
    let pbr_output = ui.get_texture_id(pbr_output);

    ui.window("Viewport")
        .size([950.0, 200.0], imgui::Condition::Once)
        .resizable(true)
        .build(|| {
            imgui::Image::new(pbr_output, ui.window_size()).build(ui);
        });

    Ok(())
}
