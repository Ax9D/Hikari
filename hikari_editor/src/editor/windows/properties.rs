use std::any::TypeId;

use crate::{components::{ComponentDispatch, EditorComponents}, imgui, editor::meta::{EditorOnly, EditorInfo}};
use hikari::core::*;
use hikari_editor::*;
use imgui::{ImguiUiExt, TreeNodeFlags};

use super::{Editor, EditorWindow};

#[derive(Default)]
pub struct Properties {
    pub position_locked: bool,
    pub scale_locked: bool,
    pub rotation_locked: bool,
}

fn component_selection(
    ui: &imgui::Ui,
    entity: Entity,
    world: &mut World,
    components: &EditorComponents,
    filtered_types: &[TypeId],
) {
    ui.popup("ComponentSelection", || {
        let mut sorted_components: Vec<_> = components
            .iter()
            .filter_map(|(type_id, component)| {
                if filtered_types.contains(type_id) {
                    None
                } else {
                    Some(component)
                }
            })
            .collect();

        sorted_components.sort_by_key(|component| component.sort_key());

        for component in sorted_components {
            if ui.selectable(component.name()) {
                component.add_component(entity, world).unwrap();
            }
        }
    });

    ui.new_line();
    ui.horizontal_align(
        || {
            if ui.button("Add Component") {
                ui.open_popup("ComponentSelection");
            }
        },
        0.5,
        ui.calc_text_size("Add Component")[0],
    );
}
fn draw_component(
    ui: &imgui::Ui,
    entity: Entity,
    component: &ComponentDispatch,
    world: &mut World,
    editor: &mut Editor,
    state: EngineState,
) -> anyhow::Result<()> {
    // let token = ui
    //                         .tree_node_config(component.name())
    //                         .opened(true, imgui::Condition::FirstUseEver)
    //                         //.frame_padding(true)
    //                         .flags(imgui::TreeNodeFlags::FRAMED)
    //                         .allow_item_overlap(true)
    //                         .push();
    //     ui.same_line_with_pos(
    //         ui.window_content_region_max()[0]
    //             - ui.window_content_region_min()[0]
    //             - ui.frame_height() / 2.0
    //             + 1.0,
    //     );
    //     let remove = ui.button_with_size("x", [ui.frame_height(), ui.frame_height()]);
    //     if let Some(_token) = token {
    //     ui.new_line();
    //     component.draw_component(ui, entity, world, editor, state)?;
    // }

    let mut open = true;

    if ui.collapsing_header_with_close_button(
        component.name(),
        TreeNodeFlags::DEFAULT_OPEN,
        &mut open,
    ) {
        component.draw_component(ui, entity, world, editor, state)?;
    }

    if !open {
        component.remove_component(entity, world).unwrap();
    }

    Ok(())
}
impl EditorWindow for Properties {
    fn draw(ui: &imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
        let filtered_types = [TypeId::of::<EditorInfo>(), TypeId::of::<EditorOnly>()];

        let result = ui
            .window("Properties")
            .size([300.0, 400.0], imgui::Condition::FirstUseEver)
            .resizable(true)
            .build(|| -> anyhow::Result<()> {
                let components = state.get::<EditorComponents>().unwrap();
                let outliner = &editor.outliner;

                if let Some(entity) = outliner.selected() {
                    let mut world = state.get_mut::<World>().unwrap();

                    {
                        hikari::dev::profile_scope!("Draw Components");
                        let _id = ui.push_id_int(entity.id() as i32);
                        let entity_ref = world.entity(entity).unwrap();

                        let entity_ty_ids = entity_ref
                            .component_types()
                            .filter(|type_id| !filtered_types.contains(type_id));

                        let mut entity_components = Vec::new();

                        for ty_id in entity_ty_ids {
                            if let Some(component) = components.get(ty_id) {
                                entity_components.push(component);
                            }
                        }
                        entity_components.sort_by_key(|component| component.sort_key());

                        for component in entity_components {
                            draw_component(ui, entity, component, &mut world, editor, state)?;
                        }
                    }
                    component_selection(ui, entity, &mut world, &components, &filtered_types);
                }
                Ok(())
            });

        if let Some(result) = result {
            result?;
        }

        Ok(())
    }
}
