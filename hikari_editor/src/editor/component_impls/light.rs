use hikari::g3d::Light;

use crate::{editor::components::EditorComponent, *};

impl EditorComponent for Light {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Light Component"
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn draw(
        &mut self,
        ui: &imgui::Ui,
        _entity: Entity,
        _editor: &mut Editor,
        _state: EngineState,
    ) -> anyhow::Result<()> {
        imgui::ColorPicker4::new("color", &mut self.color)
            .display_rgb(true)
            .display_hex(true)
            .build(ui);

        imgui::Drag::new("intensity").build(ui, &mut self.intensity);

        ui.checkbox("cast shadows", &mut self.cast_shadows);
        Ok(())
    }

    fn clone(&self) -> Self
    where
        Self: Sized,
    {
        Clone::clone(&self)
    }
}
