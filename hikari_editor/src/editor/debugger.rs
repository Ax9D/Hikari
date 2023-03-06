use super::{Editor, EditorWindow};
use hikari::imgui::*;
use hikari::{
    asset::{AssetDB, AssetManager},
    pbr::WorldRenderer,
};
use hikari_editor::EngineState;
use parking_lot::{lock_api::RwLockUpgradableReadGuard, RwLockWriteGuard};

pub struct Debugger {
    is_open: bool,
    search: String,
}
impl Debugger {
    pub fn new() -> Self {
        Self {
            is_open: false,
            search: String::new(),
        }
    }
}
fn estimate_asset_size(asset_db: &AssetDB) -> u64 {
    let mut size = 0;
    for record in asset_db.records() {
        let meta = std::fs::metadata(&record.path);
        if let Ok(meta) = meta {
            size += meta.len();
        }
    }

    size
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
fn draw_asset_db(ui: &Ui, record: &hikari::asset::Record, asset_db: &AssetDB) {
    let unref = asset_db.uuid_to_handle(&record.uuid).is_none();

    ui.disabled(unref, || {
        ui.table_next_column();
        ui.text(record.path.display().to_string());

        ui.table_next_column();
        ui.text(record.uuid.to_string());
    });
}

impl EditorWindow for Debugger {
    fn open(editor: &mut Editor) {
        editor.debugger.is_open = true;
    }
    fn is_open(editor: &mut Editor) -> bool {
        editor.debugger.is_open
    }
    fn draw(ui: &Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
        let debugger = &mut editor.debugger;

        ui.window("Debugger")
            .size([400.0, 400.0], Condition::FirstUseEver)
            .resizable(true)
            .opened(&mut debugger.is_open)
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
                            let size = estimate_asset_size(&asset_db);
                            let human_size = human_bytes(size);
                            let mut storage = ui.storage();
                            storage.insert(ui.new_id_int(ASSET_FILE_SIZE), human_size);
                        }

                        ui.text(format!("Asset Count: {}", asset_db.records().len()));

                        if let Some(size) =
                            ui.storage().get::<String>(ui.new_id_int(ASSET_FILE_SIZE))
                        {
                            ui.same_line();
                            ui.text(format!("Estimated Size: {}", size));
                        }

                        ui.input_text("##Search", &mut debugger.search)
                            .hint("Search")
                            .build();

                        if let Some(_token) = ui.begin_table_header_with_flags(
                            "AssetInfo",
                            [
                                TableColumnSetup {
                                    name: "Path",
                                    init_width_or_weight: 50.0,
                                    ..Default::default()
                                },
                                TableColumnSetup {
                                    name: "UUID",
                                    init_width_or_weight: 50.0,
                                    ..Default::default()
                                },
                            ],
                            TableFlags::BORDERS
                                | TableFlags::ROW_BG
                                | TableFlags::RESIZABLE
                                | TableFlags::SCROLL_X
                                | TableFlags::SCROLL_Y
                                | TableFlags::SIZING_STRETCH_PROP,
                        ) {
                            if debugger.search.is_empty() {
                                let clipper = ListClipper::new(asset_db.records().len() as i32);
                                let mut clipper = clipper.begin(ui);

                                while clipper.step() {
                                    for record_ix in clipper.display_start()..clipper.display_end()
                                    {
                                        let record = &asset_db.records()[record_ix as usize];
                                        draw_asset_db(ui, record, &asset_db);
                                    }
                                }
                            } else {
                                let filtered: Vec<_> = asset_db
                                    .records()
                                    .iter()
                                    .filter(|record| {
                                        let path_string = record
                                            .path
                                            .to_str()
                                            .map(|str| str.to_lowercase())
                                            .unwrap_or(String::new());

                                        path_string.contains(&debugger.search.to_lowercase())
                                    })
                                    .collect();

                                let clipper = ListClipper::new(filtered.len() as i32);
                                let mut clipper = clipper.begin(ui);

                                while clipper.step() {
                                    for record_ix in clipper.display_start()..clipper.display_end()
                                    {
                                        let record = &filtered[record_ix as usize];
                                        draw_asset_db(ui, record, &asset_db);
                                    }
                                }
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
}
