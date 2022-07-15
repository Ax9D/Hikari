use crate::{imgui, EngineState};
use hikari::core::*;

use super::{EditorComponents, Editor};

#[derive(Default)]
pub struct Properties {}

pub fn draw(ui: &imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
    let result = ui
        .window("Properties")
        .size([300.0, 400.0], imgui::Condition::Once)
        .resizable(true)
        .build(|| -> anyhow::Result<()> {
            let components = state.get::<EditorComponents>().unwrap();
            let outliner = &editor.outliner;

            if let Some(entity) = outliner.selected {
                let mut world = state.get_mut::<World>().unwrap();
                let entity_ref = world.entity(entity).unwrap();
                let type_ids = entity_ref.component_types().collect::<Vec<_>>();

                ui.popup("component_selection", || {
                    for (_, component) in components.iter() {
                        if ui.selectable(component.name()) {
                            component.add_component(entity, &mut world).unwrap();
                        }
                    }
                });

                if ui.button("+") {
                    ui.open_popup("component_selection");
                }

                {
                    hikari::dev::profile_scope!("Draw Components");
                    let _id = ui.push_id_int(entity.id() as i32);
                    for ty in type_ids {
                        if let Some(dispatch) = components.get(ty) {
                            let _token = ui
                                .tree_node_config(dispatch.name())
                                .opened(true, imgui::Condition::FirstUseEver)
                                .frame_padding(true)
                                .flags(imgui::TreeNodeFlags::FRAMED)
                                .allow_item_overlap(true)
                                .push();

                            ui.same_line_with_pos(
                                ui.window_content_region_max()[0]
                                    - ui.window_content_region_min()[0]
                                    - ui.frame_height() / 2.0
                                    + 1.0,
                            );
                            let remove =
                                ui.button_with_size("x", [ui.frame_height(), ui.frame_height()]);
                            ui.new_line();

                            if let Some(_) = _token {
                                dispatch.draw_component(ui, entity, &mut world, editor, state)?;
                            }

                            if remove {
                                dispatch.remove_component(entity, &mut world).unwrap();
                            }
                        }
                    }
                }
            }
            Ok(())
        });

    if let Some(result) = result {
        result?;
    }

    Ok(())
}
