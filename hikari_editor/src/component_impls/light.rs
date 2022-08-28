use crate::{components::EditorComponent, *};
use hikari::g3d::{Light, ShadowInfo, LightKind};
use hikari_editor::*;

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

        let mut shadows_enabled = self.shadow.is_some();

        let changed = ui.checkbox("cast shadows", &mut shadows_enabled);
        if changed {
            if shadows_enabled {
                self.shadow = Some(ShadowInfo::default());
            } else {
                self.shadow = None;
            }
        }

        if let Some(shadow_info) = &mut self.shadow {
            ui.input_float("Constant Bias", &mut shadow_info.constant_bias).build();
            imgui::Slider::new("Normal Bias", 0.0, 5.0).build(ui, &mut shadow_info.normal_bias);
            if self.kind == LightKind::Directional {
                imgui::Slider::new("Cascade Split Lambda", 0.0, 1.0).build(ui, &mut shadow_info.cascade_split_lambda);
                ui.input_float("Max Shadow Distance", &mut shadow_info.max_shadow_distance).build();
                imgui::Slider::new("Shadow Fade", 0.0, 1.0).build(ui, &mut shadow_info.fade);
            }
        }
        Ok(())
    }

    fn clone(&self) -> Self
    where
        Self: Sized,
    {
        Clone::clone(&self)
    }
}
