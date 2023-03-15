use hikari::core::*;
use hikari::g3d::{Outline, MeshRender};
use hikari::math::*;

use crate::editor::meta::{EditorInfo, EditorOnly};
use crate::widgets::RenameInput;
use hikari::imgui::*;
use hikari_editor::*;

use crate::editor::{Editor, EditorWindow};

#[derive(Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Outliner {
    #[serde(skip)]
    selected: Option<Entity>,
}
impl Outliner {
    pub fn add_entity(&mut self, world: &mut World, name: &str) -> Entity {
        let entity = world.create_entity();
        let index = world.len();
        let editor_info = EditorInfo::new(name, index);

        world.add_component(entity, editor_info).unwrap();
        entity
    }
    pub fn ordered_entities(&mut self, world: &mut World) -> Vec<(Entity, usize)> {
        hikari::dev::profile_scope!("Outliner Entity sorting");

        let mut ordered_entities = Vec::new();
        for (entity, info) in world.query_mut::<Without<&EditorInfo, &EditorOnly>>() {
            ordered_entities.push((entity, info.index));
        }

        ordered_entities.sort_by(|(_, a), (_, b)| a.cmp(b));

        ordered_entities
    }
    pub fn remove_entity(&mut self, world: &mut World, entity: Entity) -> Result<(), NoSuchEntity> {
        world.remove_entity(entity)
    }
    pub fn selected(&self) -> Option<Entity> {
        self.selected
    }
    pub fn set_selected(&mut self, entity: Entity, world: &mut World) {
        if let Some(current_entity) = self.selected {
            let _res = world.remove_component::<Outline>(current_entity);
        }
        if world.has_component::<MeshRender>(entity) {
            world.add_component(entity, Outline {
                color: Vec3::new(0.952, 0.411, 0.105),
                ..Default::default()
            }).unwrap();
        }

        if let Ok(editor_info) = world.get_component::<&EditorInfo>(entity) {
            log::debug!("Selected: {:?}, Editor Index: {}", entity, editor_info.index);
        }
        self.selected = Some(entity);
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

                if !editor.project_manager.current_scene().is_some() {
                    ui.text("No Project");
                    return;
                };

                if ui.button("+") {
                    outliner.add_entity(&mut world, "untitled");
                }

                if ui.is_window_focused() && ui.is_key_down(Key::Delete) {
                    if let Some(entity) = outliner.selected {
                        outliner.remove_entity(&mut world, entity).unwrap();
                        outliner.selected = None;
                    }
                }

                let ordered_entities;
                {
                    hikari::dev::profile_scope!("Outliner Entity sorting");
                    ordered_entities = outliner.ordered_entities(&mut world);
                }

                let mut selected = None;
                for (entity, _) in ordered_entities {
                    let mut editor_info = world.get_component::<&mut EditorInfo>(entity).unwrap();

                    let entity_id = ui.new_id(entity.id() as usize);
                    let _id = ui.push_id_int(entity.id() as i32);

                    RenameInput::new(entity_id, &mut editor_info.name).build(
                        ui,
                        rename_state,
                        |current| {
                            let clicked = ui
                                .selectable_config(&current)
                                .selected(outliner.selected() == Some(entity))
                                .build();

                            if clicked {
                                selected = Some(entity);
                            }
                        },
                    );
                }
                if let Some(selected) = selected {
                    outliner.set_selected(selected, &mut world);
                }
            });

        Ok(())
    }
}
