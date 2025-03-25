use hikari::core::*;
use hikari::g3d::{MeshRender, Outline};
use hikari::math::*;

use crate::editor::meta::{EditorOutlinerInfo};
use crate::widgets::RenameInput;
use hikari::imgui::*;
use hikari_editor::*;

use crate::editor::{Editor, EditorWindow};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Outliner {
    #[serde(skip)]
    selected: Option<Entity>,
}
impl Outliner {
    pub fn on_world_loaded(&mut self, world: &mut World) {
        self.selected = None;

        let query =  world.query_mut::<&mut EditorOutlinerInfo>();
        let outliner_component = query.into_iter().next();
        let outliner_absent = outliner_component.is_none();

        if outliner_absent {
            let mut outliner_info = EditorOutlinerInfo::default();
            //let mut order = Vec::new();

            // // For compatibility with old versions
            // for (entity, editor_info) in world.query_mut::<&EditorInfo>() {
            //     #[allow(deprecated)]
            //     order.push((entity, editor_info.index()));
            // }

            // // Sort entities by index specified in EditorInfo. This is deprecated now.
            // order.sort_by(|(_, a), (_, b)| a.cmp(b));

            //let order: Vec<_> = order.iter().map(|&(entity, _)| entity).collect();

            outliner_info.order = Vec::new();
            world.create_entity_with((outliner_info, ));
        }
    }
    pub fn add_entity(&mut self, world: &mut World, name: &str) -> Entity {
        let entity = world.create_entity_with_name(name);

        Self::outliner_info(world).order.push(entity);
        entity
    }
    pub fn duplicate_entity(&mut self, world: &mut World, entity: Entity, registry: &Registry) -> Result<(), NoSuchEntity> {
        let dup_entity = world.duplicate_entity(entity, registry)?;
        
        let outliner_info = Self::outliner_info(world);
        outliner_info.order.push(dup_entity);
        Ok(())
    }
    fn outliner_info(world: &mut World) -> &mut EditorOutlinerInfo {
        let query =  world.query_mut::<&mut EditorOutlinerInfo>();
        let (_, info) = query.into_iter().next().unwrap();

        info
    }
    pub fn ordered_entities(&mut self, world: &mut World) -> Vec<Entity> {
        hikari::dev::profile_scope!("Outliner Entity sorting");

        let outliner_info = Self::outliner_info(world);
        let mut ordered_entities = Vec::new();

        //Put entities authored by user
        for entity in &outliner_info.order {
            ordered_entities.push(*entity);
        }

        ordered_entities
    }
    pub fn remove_entity(&mut self, world: &mut World, entity: Entity) -> Result<(), NoSuchEntity> {
        world.remove_entity(entity)?;

        Self::outliner_info(world).order.retain(|&x| x != entity);

        Ok(())
    }
    pub fn selected(&self) -> Option<Entity> {
        self.selected
    }
    pub fn set_selected(&mut self, entity: Entity, world: &mut World) {
        if let Some(current_entity) = self.selected {
            let _res = world.remove_component::<Outline>(current_entity);
        }
        if world.has_component::<MeshRender>(entity) {
            world
                .add_component(
                    entity,
                    Outline::new(Vec3::new(0.952, 0.411, 0.105)),
                )
                .unwrap();
        }
        self.selected = Some(entity);
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

                if !editor.project_manager.current_world().is_some() {
                    ui.text("No World");
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
                for entity in ordered_entities {
                    let mut entity_info = world.get_component::<&mut EntityId>(entity).unwrap();

                    let entity_id = ui.new_id(entity.id() as usize);
                    let _id = ui.push_id_int(entity.id() as i32);

                    RenameInput::new(entity_id, &mut entity_info.name).build(
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
