use std::collections::HashMap;

use hikari::imgui::*;
use hikari::math::{EulerRot, Quat, Transform, Vec3};
use hikari_editor::*;

use crate::{components::EditorComponent, *};

fn rotation_controls(
    ui: &Ui,
    width: f32,
    quat: &mut Quat,
    entity: Entity,
    euler_cache: &mut HashMap<Entity, (f32, f32, f32)>,
) {
    let (x, y, z) = euler_cache
        .entry(entity)
        .or_insert_with(|| quat.to_euler(EulerRot::XYZ))
        .clone();

    let externally_changed =
        !Quat::from_euler(EulerRot::XYZ, x, y, z).abs_diff_eq(*quat, std::f32::EPSILON);

    let (x, y, z) = if externally_changed {
        quat.to_euler(EulerRot::XYZ)
    } else {
        (x, y, z)
    };

    let mut angles = Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees());

    let changed = DragVec3::new("rotation")
        .speed(0.5)
        .width(width)
        .display_format("%.3f°")
        .build(ui, &mut angles);

    if changed {
        let (x, y, z) = (
            angles.x.to_radians(),
            angles.y.to_radians(),
            angles.z.to_radians(),
        );
        *euler_cache.get_mut(&entity).unwrap() = (x, y, z);
        *quat = Quat::from_euler(EulerRot::XYZ, x, y, z);
    }
}

impl EditorComponent for Transform {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Transform Component"
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        Transform::default()
    }

    fn draw(
        &mut self,
        ui: &Ui,
        entity: Entity,
        editor: &mut Editor,
        _state: EngineState,
    ) -> anyhow::Result<()> {
        hikari::dev::profile_scope!("Transform Component");
        let mut storage = ui.storage();
        //FIXME: Memory leak when entities get deleted
        let euler_cache = storage.get_or_insert_with(ui.new_id_str("euler_cache"), || {
            HashMap::<Entity, (f32, f32, f32)>::new()
        });

        // let mut position: [f32; 3] = self.position.into();
        // imgui::Drag::new("position")
        //     .speed(0.1)
        //     .build_array(ui, &mut position);
        // self.position = position.into();

        let width = ui.content_region_avail()[0];

        DragVec3::new("position")
            .speed(0.1)
            .width(width)
            .build(ui, &mut self.position);

        rotation_controls(ui, width, &mut self.rotation, entity, euler_cache);

        DragVec3::new("scale")
            .speed(0.03)
            .width(width)
            .reset(1.0)
            .range(0.0, f32::MAX)
            .proportional(editor.properties.scale_locked)
            .build(ui, &mut self.scale);

        ui.same_line_with_spacing(0.0, 5.0);
        ui.checkbox("##ScaleLock", &mut editor.properties.scale_locked);

        Ok(())
    }

    fn sort_key() -> usize {
        0
    }
}
