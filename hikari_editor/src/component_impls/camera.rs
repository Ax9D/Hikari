use crate::components::EditorComponent;
use hikari::g3d::*;
use hikari::imgui::*;
use hikari_editor::*;

impl EditorComponent for Camera {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Camera Component"
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn draw(
        &mut self,
        ui: &Ui,
        _entity: hikari::core::Entity,
        _editor: &mut crate::editor::Editor,
        _state: EngineState,
    ) -> anyhow::Result<()> {
        Drag::new("near")
            .range(0.0, f32::MAX)
            .build(ui, &mut self.near);
        Drag::new("far")
            .range(0.0, f32::MAX)
            .build(ui, &mut self.far);

        match &mut self.projection {
            Projection::Perspective(fov) => {
                Drag::new("fov").range(0.0, 360.0).build(ui, fov);
            }
            Projection::Orthographic => todo!(),
        }

        Drag::new("exposure")
            .range(0.0, f32::MAX)
            .build(ui, &mut self.exposure);
        Ok(())
    }
}
