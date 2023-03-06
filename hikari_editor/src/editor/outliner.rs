use hikari::core::*;
use hikari::math::*;

use crate::widgets::RenameInput;
use hikari::imgui::*;
use hikari_editor::*;

use super::meta::{EditorInfo, EditorOnly};
use super::{Editor, EditorWindow};

#[derive(Default)]
pub struct Outliner {
    pub selected: Option<Entity>,
}
impl Outliner {
    pub fn add_entity(&mut self, world: &mut World, name: &str) -> Entity {
        let index = world.len();
        let entity = world.create_entity();

        world
            .add_component(
                entity,
                EditorInfo {
                    name: name.to_owned(),
                    index,
                },
            )
            .unwrap();
        entity
    }
    pub fn remove_entity(&mut self, world: &mut World, entity: Entity) -> Result<(), NoSuchEntity> {
        world.remove_entity(entity)
    }
    pub fn reset(&mut self) {
        self.selected = None;
    }
}
impl EditorWindow for Outliner {
    fn draw(ui: &Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
        let mut world = state.get_mut::<World>().unwrap();

        ui.window("Outliner")
            .size([300.0, 400.0], Condition::FirstUseEver)
            .resizable(true)
            .build(|| {
                let outliner = &mut editor.outliner;
                let rename_state = &mut editor.rename_state;

                if ui.button("+") {
                    outliner.add_entity(&mut world, "untitled");
                }

                if ui.is_window_focused() && ui.is_key_down(Key::Delete) {
                    if let Some(entity) = outliner.selected {
                        outliner.remove_entity(&mut world, entity).unwrap();
                        outliner.selected = None;
                    }
                }

                let mut ordered_entities;
                {
                    hikari::dev::profile_scope!("Outliner Entity sorting");
                    ordered_entities = Vec::with_capacity(world.len());
                    for (entity, info) in world.query_mut::<Without<&EditorInfo, &EditorOnly>>() {
                        ordered_entities.push((entity, info.index));
                    }

                    ordered_entities.sort_by(|(_, a), (_, b)| a.cmp(b));
                }

                for (entity, _) in ordered_entities {
                    let mut editor_info = world.get_component::<&mut EditorInfo>(entity).unwrap();
                    let entity_id = ui.new_id_int(entity.id() as i32);
                    let _id = ui.push_id_int(entity.id() as i32);

                    RenameInput::new(entity_id, &mut editor_info.name).build(
                        ui,
                        rename_state,
                        |current| {
                            let clicked = ui
                                .selectable_config(&current)
                                .selected(outliner.selected == Some(entity))
                                .build();

                            if clicked {
                                outliner.selected = Some(entity);
                            }
                        },
                    );
                }
            });

        Ok(())
    }
}
