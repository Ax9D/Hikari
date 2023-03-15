use hikari::core::Time;
use hikari::g3d::ShaderLibrary;
use hikari::g3d::ShaderLibraryConfig;
use hikari::pbr::ShadowResolution;
use hikari::pbr::WorldRenderer;
use hikari::render::Gfx;
use hikari_editor::EngineState;

use super::Editor;
use super::EditorWindow;

use hikari::imgui;

pub struct RenderSettings;

impl EditorWindow for RenderSettings {
    fn draw(ui: &imgui::Ui, _editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
        ui.window("Render Settings")
            .size([300.0, 400.0], imgui::Condition::FirstUseEver)
            .resizable(true)
            .build(|| {
                let mut renderer = state.get_mut::<WorldRenderer>().unwrap();
                let mut gfx = state.get_mut::<Gfx>().unwrap();
                let mut shader_lib = state.get_mut::<ShaderLibrary>().unwrap();
                let time = state.get::<Time>().unwrap();

                renderer
                    .update_settings(&mut gfx, &mut shader_lib, |settings| {
                        let dt = time.dt();
                        ui.text(format!(
                            "Frametime: {} ms FPS: {}",
                            (dt * 1000.0).round(),
                            (1.0 / dt).round()
                        ));
                        ui.checkbox("VSync", &mut settings.vsync);
                        ui.checkbox("FXAA", &mut settings.fxaa);

                        //FIX ME: Enum selection macro maybe?
                        let mut current_res = settings.directional_shadow_map_resolution as usize;
                        ui.combo(
                            "Directional Shadow Map Resolution",
                            &mut current_res,
                            &[
                                ShadowResolution::D256,
                                ShadowResolution::D512,
                                ShadowResolution::D1024,
                                ShadowResolution::D2048,
                                ShadowResolution::D4096,
                            ],
                            |kind| match kind {
                                ShadowResolution::D256 => std::borrow::Cow::Borrowed("256"),
                                ShadowResolution::D512 => std::borrow::Cow::Borrowed("512"),
                                ShadowResolution::D1024 => std::borrow::Cow::Borrowed("1024"),
                                ShadowResolution::D2048 => std::borrow::Cow::Borrowed("2048"),
                                ShadowResolution::D4096 => std::borrow::Cow::Borrowed("4096"),
                            },
                        );

                        settings.directional_shadow_map_resolution = match current_res {
                            0 => ShadowResolution::D256,
                            1 => ShadowResolution::D512,
                            2 => ShadowResolution::D1024,
                            3 => ShadowResolution::D2048,
                            4 => ShadowResolution::D4096,
                            _ => unreachable!(),
                        };
                        ui.separator();

                        ui.checkbox("Wireframe", &mut settings.debug.wireframe);
                        ui.checkbox(
                            "Show Shadow Cascades",
                            &mut settings.debug.show_shadow_cascades,
                        );
                    })
                    .expect("Failed to update settings");

                let mut generate_debug_info = shader_lib.config().generate_debug_info;

                if ui.checkbox("Generate Shader Debug Info", &mut generate_debug_info) {
                    match shader_lib.set_generate_debug(ShaderLibraryConfig {
                        generate_debug_info,
                    }) {
                        Ok(_) => {}
                        Err(why) => log::error!("{}", why),
                    }
                }

                if ui.button("Reload Shaders") {
                    match shader_lib.reload() {
                        Ok(_) => {}
                        Err(why) => log::error!("{}", why),
                    }
                }
            });

        Ok(())
    }
}
