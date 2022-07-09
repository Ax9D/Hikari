use std::collections::HashMap;

use hikari::core::*;
use hikari::input::KeyCode;
use hikari::math::*;
use imgui::ImguiUiExt;

use crate::{imgui, EngineState};

use super::Editor;
use super::RenameState;

#[derive(Clone)]
struct EditorInfo {
    name: String,
    index: usize
}

#[derive(Default)]
pub struct Outliner {
    pub selected: Option<Entity>,
}
impl Outliner {
    pub fn add_entity(&mut self, world: &mut World, name: &str) -> Entity {
        let index = world.len();
        let entity = world.create_entity();

        world.add_component(entity, EditorInfo {
            name: name.to_owned(),
            index
        }).unwrap();
        entity
    }
    pub fn remove_entity(&mut self, world: &mut World, entity: Entity) -> Result<(), NoSuchEntity> {
        world.remove_entity(entity)
    }
}
pub fn draw(ui: &imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
    let mut world = state.get_mut::<World>().unwrap();

    ui.window("Outliner")
        .size([300.0, 400.0], imgui::Condition::Once)
        .resizable(true)
        .build(|| {
            let outliner = &mut editor.outliner;
            let rename_state = &mut editor.rename_state;

            if ui.button("+") {
                outliner.add_entity(&mut world, "untitled");
            }

            if ui.is_window_focused() && ui.io().keys_down[KeyCode::Delete as usize] {
                if let Some(entity) = outliner.selected {
                    outliner.remove_entity(&mut world, entity).unwrap();
                    outliner.selected = None;
                }
            }

            let mut ordered_entities = vec![Entity::DANGLING; world.len()];

            world.entities().for_each(|entity_ref| {
                let order = entity_ref.get::<EditorInfo>().unwrap().index;
                ordered_entities[order] = entity_ref.entity();
            });
        
            for entity in ordered_entities {
                let mut editor_info = world.get_component_mut::<EditorInfo>(entity).unwrap();
                let entity_id = imgui::Id::Int(entity.id() as i32, ui);
                let _id = ui.push_id_int(entity.id() as i32);

                match rename_state {
                    RenameState::Renaming(id, current_name, starting_frame) if *id == entity_id => {
                        let _frame = ui.push_style_var(imgui::StyleVar::FramePadding([0.0, 0.0]));

                        let input_text_ended = ui
                        .input_text("###rename", current_name)
                        .enter_returns_true(true)
                        .build();

                        let end_rename = input_text_ended
                            // If lost focus
                            || !ui.is_window_focused()
                            // If the mouse is clicked on anything but the input text
                            || (ui.is_mouse_clicked(imgui::MouseButton::Left) && !ui.is_item_clicked());

                        //If the rename was started last frame
                        if ui.frame_count() == *starting_frame + 1 {
                            ui.set_keyboard_focus_here();
                        }

                        //If escape was pressed cancel the rename
                        let cancel_rename = ui.io().keys_down[KeyCode::Escape as usize];

                        if cancel_rename {
                            *rename_state = RenameState::Idle;
                        } else if end_rename {
                            editor_info.name = current_name.clone();
                            *rename_state = RenameState::Idle;
                        }
                    }
                    _ => {
                        let clicked = ui
                            .selectable_config(&editor_info.name)
                            .selected(outliner.selected == Some(entity))
                            .build();

                        if clicked {
                            outliner.selected = Some(entity);
                        }
                    }
                };

                if ui.is_item_focused()
                    && (ui.io().keys_down[KeyCode::F2 as usize]
                        || ui.is_double_click(imgui::MouseButton::Left))
                {
                    match rename_state {
                        RenameState::Idle => {
                            *rename_state = RenameState::Renaming(
                                entity_id,
                                editor_info.name.clone(),
                                ui.frame_count(),
                            );
                        }
                        _ => {}
                    }
                }
            }
        });

    Ok(())
}
