use std::collections::HashMap;

use hikari::math::{EulerRot, Quat, Transform, Vec3};
use hikari_editor::*;
use imgui::StorageExt;

use crate::{components::EditorComponent, *};

fn rotation_controls(
    ui: &imgui::Ui,
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

    let mut angles = [x.to_degrees(), y.to_degrees(), z.to_degrees()];

    let changed = imgui::Drag::new("rotation")
        .speed(0.5)
        .build_array(ui, &mut angles);

    if changed {
        let (x, y, z) = (
            angles[0].to_radians(),
            angles[1].to_radians(),
            angles[2].to_radians(),
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
        ui: &hikari_imgui::Ui,
        entity: Entity,
        _editor: &mut Editor,
        _state: EngineState,
    ) -> anyhow::Result<()> {
        hikari::dev::profile_scope!("Transform Component");
        let mut storage = ui.storage();
        //FIXME: Memory leak when entities get deleted
        let euler_cache = storage.get_or_insert_with(imgui::Id::Str("euler_cache", ui), || {
            HashMap::<Entity, (f32, f32, f32)>::new()
        });

        let mut position: [f32; 3] = self.position.into();
        imgui::Drag::new("position")
            .speed(0.1)
            .build_array(ui, &mut position);
        self.position = position.into();

        rotation_controls(ui, &mut self.rotation, entity, euler_cache);

        let mut scale: [f32; 3] = self.scale.into();
        imgui::Drag::new("scale")
            .speed(0.2)
            .build_array(ui, &mut scale);
        self.scale = Vec3::from(scale);

        Ok(())
    }

    fn clone(&self) -> Self
    where
        Self: Sized,
    {
        *self
    }
}
