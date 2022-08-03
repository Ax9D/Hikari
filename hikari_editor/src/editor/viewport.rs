use crate::imgui;
use crate::imgui::gizmo::*;
use hikari::{pbr::WorldRenderer, render::imgui_support::TextureExt};
use hikari_editor::*;

use super::Editor;

#[derive(Default)]
pub struct Viewport {
    _gizmo_context: GizmoContext
}
pub fn draw(ui: &imgui::Ui, _editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
    //let mut viewport = &mut editor.viewport;
    //let outliner = &mut editor.outliner;
    
    let mut renderer = state.get_mut::<WorldRenderer>().unwrap();
    //let mut world = state.get_mut::<World>().unwrap();
    
    ui.window("Viewport")
        .size([950.0, 200.0], imgui::Condition::Once)
        .resizable(true)
        .build(|| {
            let window_size_float = ui.content_region_avail();
            let window_size = (
                window_size_float[0].round() as u32,
                window_size_float[1].round() as u32,
            );
            let renderer_size = renderer.size();

            if window_size != renderer_size {
                renderer.set_viewport(window_size.0, window_size.1);
            }

            let pbr_output = renderer.get_output_image();
            let pbr_output = ui.get_texture_id(pbr_output);
            imgui::Image::new(pbr_output, window_size_float).build(ui);

            // if let Some(entity) = outliner.selected {
            //     let mut transform = world.get_component_mut::<Transform>(entity).unwrap();
            //     world.get_component_mut(entity)
            //     viewport.gizmo_context.gizmo(ui)
            //     .operation(Operation::Translate)
            //     .manipulate(*transform, projection, view);
            // }
        });

    Ok(())
}
