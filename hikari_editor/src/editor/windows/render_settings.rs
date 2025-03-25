use hikari::core::Time;
use hikari::g3d::ShaderLibrary;
use hikari::g3d::ShaderLibraryConfig;
use hikari::pbr::DebugView;
use hikari::pbr::ShadowResolution;
use hikari::pbr::WorldRenderer;
use hikari::render::Gfx;
use hikari_editor::EngineState;
use serde::Deserialize;
use serde::Serialize;

use super::Editor;
use super::EditorWindow;

use hikari::imgui;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderSettings {
    open: bool
}

impl EditorWindow for RenderSettings {
    fn is_open(editor: &mut Editor) -> bool {
        editor.render_settings.open
    }
    fn open(editor: &mut Editor) {
        editor.render_settings.open = true;
    }
    fn draw(ui: &imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
        ui.window("Render Settings")
            .size([300.0, 400.0], imgui::Condition::FirstUseEver)
            .opened(&mut editor.render_settings.open)
            .resizable(true)
            .build(|| {
                let mut renderer = state.get_mut::<WorldRenderer>().unwrap();
                let mut gfx = state.get_mut::<Gfx>().unwrap();
                let mut shader_lib = state.get_mut::<ShaderLibrary>().unwrap();
                let time = state.get::<Time>().unwrap();

                let resolutions = [
                    (640, 360),
                    (800, 600),
                    (1280, 720),
                    (1600, 900),
                    (1920, 1080),
                    (3840, 2160),
                ];
                let render_res = renderer.size();
                let mut render_res_item = resolutions.iter().position(|&res| res == render_res).unwrap_or(4);

                let changed = ui.combo(
                    "Render Resolution",
                    &mut render_res_item,
                    &resolutions,
                    |kind|
                        format!("{}x{}", kind.0, kind.1).into()
                );
                if changed {
                    let (width, height) = resolutions[render_res_item];
                    renderer.resize(width, height).unwrap();
                }        

                let dt = time.dt();
                ui.text(format!(
                    "Frametime: {} ms FPS: {}",
                    (dt * 1000.0).round(),
                    (1.0 / dt).round()
                ));

                renderer
                    .update_settings(&mut gfx, &mut shader_lib, |settings| {

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

                        let mut current_view = settings.debug.view as usize;
                        ui.combo(
                            "Debug View",
                            &mut current_view,
                            &[
                                DebugView::None,
                                DebugView::Unlit,
                                DebugView::Wireframe,
                            ],
                            |kind| match kind {
                                DebugView::None => std::borrow::Cow::Borrowed("None"),
                                DebugView::Unlit => std::borrow::Cow::Borrowed("Unlit"),
                                DebugView::Wireframe => std::borrow::Cow::Borrowed("Wireframe"),
                            },
                        );

                        settings.debug.view = match current_view {
                            0 => DebugView::None,
                            1 => DebugView::Unlit,
                            2 => DebugView::Wireframe,
                            _ => unreachable!(),
                        };

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
