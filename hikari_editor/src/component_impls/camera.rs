use crate::components::EditorComponent;
use hikari::{g3d::*};
use hikari_editor::*;
use hikari_imgui::*;

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
        ui: &hikari_imgui::Ui,
        _entity: hikari::core::Entity,
        _editor: &mut crate::editor::Editor,
        _state: EngineState,
    ) -> anyhow::Result<()> {
        Drag::new("near").build(ui, &mut self.near);
        Drag::new("far").build(ui, &mut self.far);

        match &mut self.projection {
            Projection::Perspective(fov) => {
                Drag::new("fov").build(ui, fov);
            }
            Projection::Orthographic => todo!(),
        }

        Drag::new("exposure").build(ui, &mut self.exposure);
        Ok(())
    }

    fn clone(&self) -> Self
    where
        Self: Sized,
    {
        Clone::clone(&self)
    }
}
