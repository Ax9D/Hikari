use crate::{
    components::EditorComponent,
    editor::meta::{EditorInfo, EditorOnly},
    *,
};
use hikari_editor::*;

impl EditorComponent for EditorInfo {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Editor Info"
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        unimplemented!("Editor Only Component, Don't call new")
    }

    fn draw(
        &mut self,
        _ui: &imgui::Ui,
        _entity: Entity,
        _editor: &mut Editor,
        _state: EngineState,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

impl EditorComponent for EditorOnly {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Editor Only Tag"
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn draw(
        &mut self,
        _ui: &imgui::Ui,
        _entity: Entity,
        _editor: &mut Editor,
        _state: EngineState,
    ) -> anyhow::Result<()> {
        unimplemented!()
    }
}

// impl EditorComponent for EditorEntityInfo {
//     fn name() -> &'static str
//     where
//         Self: Sized {
//         "Editor Entity Info"
//     }

//     fn new() -> Self
//     where
//         Self: Sized {
//         unimplemented!("Editor Only Component, Don't call new")
//     }

//     fn draw(
//         &mut self,
//         ui: &imgui::Ui,
//         entity: Entity,
//         editor: &mut Editor,
//         state: EngineState,
//     ) -> anyhow::Result<()> {
//         unimplemented!()
//     }

//     fn clone(&self) -> Self
//     where
//         Self: Sized {
//         Clone::clone(&self)
//     }
// }
