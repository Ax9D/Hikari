use crate::editor::components::EditorComponent;
use hikari::{g3d::*, pbr::WorldRenderer};
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
        state: crate::EngineState,
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

        let mut renderer = state.get_mut::<WorldRenderer>().unwrap();
        let settings = renderer.settings();
        //ui.checkbox("vsync", &mut settings.vsync);
        ui.checkbox("fxaa", &mut settings.fxaa);
        Ok(())
    }

    fn clone(&self) -> Self
    where
        Self: Sized,
    {
        Clone::clone(&self)
    }
}
