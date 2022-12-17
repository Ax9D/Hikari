use super::Editor;
use crate::imgui;
use hikari::{
    asset::{AssetDB, AssetManager},
    pbr::WorldRenderer,
};
use hikari_editor::*;
use imgui::{StorageExt, TableColumnSetup, TableFlags};
use parking_lot::{lock_api::RwLockUpgradableReadGuard, RwLockWriteGuard};

pub struct Debugger {
    is_open: bool,
}
impl Debugger {
    pub fn new() -> Self {
        Self { is_open: false }
    }
    pub fn open(&mut self) {
        self.is_open = true;
    }
}
fn estimate_asset_size(asset_db: &AssetDB) -> std::io::Result<u64> {
    let mut size = 0;
    for record in asset_db.records() {
        let meta = std::fs::metadata(&record.path)?;
        size += meta.len();
    }

    Ok(size)
}
// https://git.sr.ht/~f9/human_bytes/tree/main/item/src/lib.rs
fn human_bytes(size: u64) -> String {
    let size = size as f64;

    const UNIT: f64 = 1024.0;
    const SUFFIX: [&str; 9] = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

    if size <= 0.0 {
        return "0 B".to_string();
    }

    let base = size.log10() / UNIT.log10();

    let result = format!("{:.1}", UNIT.powf(base - base.floor()),)
        .trim_end_matches(".0")
        .to_owned();

    // Add suffix
    [&result, SUFFIX[base.floor() as usize]].join(" ")
}
pub fn draw(ui: &imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
    if !editor.debugger.is_open {
        return Ok(());
    }

    ui.window("Debugger")
        .size([400.0, 400.0], imgui::Condition::Once)
        .resizable(true)
        .opened(&mut editor.debugger.is_open)
        .build(|| -> anyhow::Result<()> {
            if let Some(_token) = ui.tab_bar("Render Graph Tabs") {
                let renderer = state.get::<WorldRenderer>().unwrap();

                if let Some(_token) = ui.tab_item("Images") {
                    let resources = renderer.graph_resources();
                    let images = resources.image_handles();
                    for (name, handle) in images {
                        let image = resources.get_image(handle).unwrap();
                        if let Some(_token) = ui.tree_node(&name) {
                            ui.text(format!("VkImage: {:?}", image.image()));
                            ui.text(format!("Config: {:#?}", image.config()));
                        }
                    }
                }

                if let Some(_token) = ui.tab_item("Asset DB") {
                    let asset_manager = state.get::<AssetManager>().unwrap();

                    if ui.button("Save") {
                        asset_manager.save_db()?;
                    }

                    ui.same_line();

                    let asset_db = asset_manager.asset_db().upgradable_read();

                    let asset_db = if ui.button("Clean Unref") {
                        let mut writer = RwLockUpgradableReadGuard::upgrade(asset_db);
                        writer.clean_unref();
                        RwLockWriteGuard::downgrade(writer)
                    } else {
                        RwLockUpgradableReadGuard::downgrade(asset_db)
                    };
                    if ui.is_item_hovered() {
                        ui.tooltip_text(
                            "Removes all assets which aren't referred to by any other asset",
                        );
                    }

                    ui.same_line();

                    const ASSET_FILE_SIZE: i32 = 0xDEE;
                    if ui.button("Estimate Size") {
                        let size = estimate_asset_size(&asset_db)?;
                        let human_size = human_bytes(size);
                        let mut storage = ui.storage();
                        storage.insert(imgui::Id::Int(ASSET_FILE_SIZE, ui), human_size);
                    }

                    ui.text(format!("Asset Count: {}", asset_db.records().len()));

                    if let Some(size) = ui
                        .storage()
                        .get::<String>(imgui::Id::Int(ASSET_FILE_SIZE, ui))
                    {
                        ui.same_line();
                        ui.text(format!("Estimated Size: {}", size));
                    }

                    if let Some(_token) = ui.begin_table_header_with_flags(
                        "AssetInfo",
                        [TableColumnSetup::new("Path"), TableColumnSetup::new("UUID")],
                        TableFlags::BORDERS
                            | TableFlags::ROW_BG
                            | TableFlags::RESIZABLE
                            | TableFlags::SCROLL_X
                            | TableFlags::SCROLL_Y,
                    ) {
                        ui.table_next_row();

                        for record in asset_db.records() {
                            let unref = asset_db.uuid_to_handle(&record.uuid).is_none();

                            ui.disabled(unref, || {
                                ui.table_next_column();
                                ui.text(record.path.display().to_string());

                                ui.table_next_column();
                                ui.text(record.uuid.to_string());
                            });
                        }
                    }
                }

                // if let Some(_token) = ui.tab_item("Render Target Debug") {
                //     ui.text("Shadow Map Atlas");
                //     let shadow_map = renderer.graph_resources().get_image_by_name("ShadowMapAtlasDebug").unwrap();

                //     imgui::Image::new(ui.get_texture_id(shadow_map), [400.0 * 4.0, 400.0]).build(ui);

                //     ui.text("Z Prepass");
                //     let depth_map = renderer.graph_resources().get_image_by_name("PrepassDepthDebug").unwrap();

                //     imgui::Image::new(ui.get_texture_id(depth_map), [400.0, 400.0]).build(ui);
                // }
            }
            Ok(())
        })
        .unwrap_or(Ok(()))
}
