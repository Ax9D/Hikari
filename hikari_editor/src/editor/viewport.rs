use hikari::{ pbr::WorldRenderer, render::imgui_support::TextureExt,
};

use crate::{imgui, EngineState};

use super::Editor;

#[derive(Default)]
pub struct Viewport;

pub fn draw(ui: &imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
    let mut renderer = state.get_mut::<WorldRenderer>().unwrap();
    ui.window("Viewport")
        .size([950.0, 200.0], imgui::Condition::Once)
        .resizable(true)
        .build(|| {
            let window_size_float = ui.content_region_avail();
            let window_size = (window_size_float[0].round() as u32, window_size_float[1].round() as u32);
            let renderer_size = renderer.size();


            if window_size != renderer_size {
                renderer.resize(window_size.0, window_size.1).expect("Failed to resize World Renderer");
            }
            let pbr_output = renderer.get_output_image();
            let pbr_output = ui.get_texture_id(pbr_output);
            imgui::Image::new(pbr_output, window_size_float).build(ui);
        });

    Ok(())
}
