use crate::{components::EditorComponent, *};
use hikari::{
    asset::AssetManager,
    g3d::{MeshRender, MeshSource, Scene},
};
use hikari_editor::*;

impl EditorComponent for MeshRender {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Mesh Render Component"
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        MeshRender {
            source: g3d::MeshSource::None,
        }
    }

    fn draw(
        &mut self,
        ui: &imgui::Ui,
        _entity: Entity,
        _editor: &mut Editor,
        state: EngineState,
    ) -> anyhow::Result<()> {
        let ass_man = state.get::<AssetManager>().unwrap();

        let mut path = if let MeshSource::Scene(scene, _) = &self.source {
            let db = ass_man.asset_db().read();
            let erased = scene.clone_erased_as_weak();
            let path = db.handle_to_path(&erased).unwrap();

            path.display().to_string()
        } else {
            "None".into()
        };
        ui.input_text("Asset", &mut path).build();
        ui.same_line();
        if ui.button("/") {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("GLTF", &["gltf", "glb"])
                .pick_file()
            {
                assert!(path.extension().is_some());
                let path = path.strip_prefix(ass_man.get_asset_dir())?;
                let scene = ass_man.load::<Scene>(path, None, false)?;
                self.source = MeshSource::Scene(scene, 0);
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
