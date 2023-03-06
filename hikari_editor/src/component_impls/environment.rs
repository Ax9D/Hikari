use hikari::{g3d::{Environment, EnvironmentTexture}, asset::AssetManager};
use hikari::imgui::*;

use crate::components::EditorComponent;

impl EditorComponent for Environment {
    fn name() -> &'static str
    where
        Self: Sized {
        "Environment Component"
    }

    fn new() -> Self
    where
        Self: Sized {
        Environment::default()
    }

    fn draw(
        &mut self,
        ui: &Ui,
        _entity: hikari::core::Entity,
        _editor: &mut crate::editor::Editor,
        state: hikari_editor::EngineState,
    ) -> anyhow::Result<()> {
        let ass_man = state.get::<AssetManager>().unwrap();

        let mut path = if let Some(handle) = &self.texture {
            let db = ass_man.asset_db().read();
            let erased = handle.clone_erased_as_weak();
            let path = db.handle_to_path(&erased).unwrap();

            path.display().to_string()
        } else {
            "None".into()
        };

        ui.input_text("Asset", &mut path).build();
        ui.same_line();
        if ui.button("/") {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("HDR Map", &hikari::g3d::SUPPORTED_ENV_TEXTURE_EXTENSIONS)
                .pick_file()
            {
                assert!(path.extension().is_some());
                let path = path.strip_prefix(ass_man.get_asset_dir())?;
                let texture = ass_man.load::<EnvironmentTexture>(path, None, false)?;
                self.texture = Some(texture);
            }
        }
        Drag::new("Intensity")
        .range(0.0, f32::MAX)
        .speed(0.01)
        .build(ui, &mut self.intensity);
    
        ui.checkbox("Use Proxy", &mut self.use_proxy);
        ui.disabled(!self.use_proxy, || {
            ui.slider("Mip Level", 0, 9, &mut self.mip_level);
        });

        Ok(())
    }

    fn clone(&self) -> Self
    where
        Self: Sized {
        <Self as Clone>::clone(self)
    }
}