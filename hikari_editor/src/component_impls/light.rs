use crate::{components::EditorComponent, *};
use hikari::g3d::{Light, LightKind};
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
        // imgui::ColorPicker4::new("color", &mut self.color)
        //     .display_rgb(true)
        //     .display_hex(true)
        //     .small_preview(true)
        //     .build(ui);
        ui.color_edit4_config("Color", &mut self.color)
            .picker(true)
            .build();

        imgui::Drag::new("Intensity")
            .range(0.0, f32::MAX)
            .speed(0.25)
            .build(ui, &mut self.intensity);

        ui.checkbox("Cast shadows", &mut self.shadow.enabled);

        ui.combo(
            "Type",
            &mut 0,
            &[LightKind::Directional],
            |kind| match kind {
                LightKind::Point => std::borrow::Cow::Borrowed("Point"),
                LightKind::Directional => std::borrow::Cow::Borrowed("Directional"),
            },
        );

        ui.enabled(self.shadow.enabled, || {
            let shadow_info = &mut self.shadow;
            ui.input_float("Sloped Scaled Bias", &mut shadow_info.slope_scaled_bias)
                .build();
            ui.slider_config("Normal Bias", 0.0, 5.0)
                .build(&mut shadow_info.normal_bias);

            if self.kind == LightKind::Directional {
                ui.slider_config("Cascade Split Lambda", 0.0, 1.0)
                    .build(&mut shadow_info.cascade_split_lambda);
                ui.input_float("Max Shadow Distance", &mut shadow_info.max_shadow_distance)
                    .build();
                ui.slider_config("Shadow Fade", 0.0, 1.0)
                    .build(&mut shadow_info.fade);
                ui.checkbox("Cull Front Face", &mut shadow_info.cull_front_face);
            }
        });
        Ok(())
    }
}
