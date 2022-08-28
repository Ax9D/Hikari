use hikari::g3d::ShaderLibrary;
use hikari::pbr::WorldRenderer;
use hikari::render::Gfx;
use hikari::render::imgui_support::TextureExt;
use hikari_editor::EngineState;

use super::Editor;
use super::imgui;

pub fn draw(ui: &imgui::Ui, _editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
    ui.window("Render Settings")        
    .size([300.0, 400.0], imgui::Condition::Once)
    .resizable(true)
    .build(|| {
        let mut renderer = state.get_mut::<WorldRenderer>().unwrap();
        let mut gfx = state.get_mut::<Gfx>().unwrap();

        let setttings = renderer.settings();

        ui.checkbox("FXAA", &mut setttings.fxaa);
        ui.separator();
        if ui.button("Disable Vsync") {
            gfx.set_vsync(false);
        }

        ui.checkbox("Show Shadow Cascades", &mut setttings.debug.show_shadow_cascades);
        
        let shadow_map = renderer.graph_resources().get_image_by_name("DirectionalShadowMapDebug0").unwrap();

        imgui::Image::new(ui.get_texture_id(shadow_map), [400.0, 400.0]).build(ui);
        ui.same_line();

        let depth_map = renderer.graph_resources().get_image_by_name("PrepassDepthDebug").unwrap();

        imgui::Image::new(ui.get_texture_id(depth_map), [400.0, 400.0]).build(ui);


        if ui.button("Reload Shaders") {
            let mut shader_lib = state.get_mut::<ShaderLibrary>().unwrap();
            match shader_lib.reload() {
                Ok(_) => {},
                Err(why) => log::error!("{}", why),
            }
        }
    });

    Ok(())
}