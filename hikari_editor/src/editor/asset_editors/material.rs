use crate::{editor::EditorWindow, Editor, widgets::{AssetSelector}};
use hikari::{
    asset::{AssetManager, Handle},
    g3d::{Material, Texture2D}, render::imgui_support::TextureExt,
};
use hikari_editor::EngineState;
use hikari::imgui::*;

#[derive(Default)]
pub struct MaterialEditor {
    is_open: bool,
    current: Option<Handle<Material>>,
    #[allow(unused)]
    changed: bool
}
impl MaterialEditor {
    #[allow(unused)]
    pub fn set_material(&mut self, material: Handle<Material>) {
        self.current = Some(material);
        self.changed = false;
    }
}
impl EditorWindow for MaterialEditor {
    fn is_open(editor: &mut Editor) -> bool {
        editor.material_editor.is_open
    }
    fn open(editor: &mut Editor) {
        editor.material_editor.is_open = true;
    }
    fn draw(ui: &Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
        let asset_manager = state.get::<AssetManager>().unwrap();
        let material_editor = &mut editor.material_editor;

        let mut outer_result = Ok(());

        ui.window("Material Editor")
            .size([800.0, 450.0], Condition::FirstUseEver)
            .resizable(true)
            .opened(&mut material_editor.is_open)
            .build(|| {
                // if let Some(_token) = ui.begin_table_with_sizing("MaterialEditorPanes", 2, TableFlags::SIZING_STRETCH_PROP, [0.0, 0.0], 0.0) {
                //     ui.table_setup_column_with(TableColumnSetup {
                //         name: "",
                //         init_width_or_weight: 50.0,
                //         ..Default::default()
                //     });

                //     ui.table_setup_column_with(TableColumnSetup {
                //         name: "",
                //         init_width_or_weight: 50.0,
                //         ..Default::default()
                //     });

                //     ui.table_next_row();
                //     ui.table_next_column();

                //     ui.text_disabled("Preview, Doesn't Do Anything for now...");

                //     ui.table_next_column();
                outer_result = material_edit(ui, &mut material_editor.current, &asset_manager);
                // }

        });


        outer_result
    }
}
fn parameter_edit(
    ui: &Ui,
    name: impl AsRef<str>,
    texture: &mut Option<Handle<Texture2D>>,
    other: impl FnOnce(),
    asset_manager: &AssetManager,
) -> anyhow::Result<()> {
    let texture_extensions = hikari::g3d::SUPPORTED_TEXTURE_EXTENSIONS;
    let _id_token = ui.push_id(name.as_ref());
    if ui.collapsing_header(name.as_ref(), TreeNodeFlags::DEFAULT_OPEN) {
        if let Some(_table_token) = ui.begin_table_with_sizing(
            "##Table",
            2,
            TableFlags::SIZING_STRETCH_PROP,
            [0.0, 0.0],
            0.0,
        ) {
            ui.table_setup_column_with(TableColumnSetup {
                name: "",
                init_width_or_weight: 20.0,
                ..Default::default()
            });

            ui.table_setup_column_with(TableColumnSetup {
                name: "",
                init_width_or_weight: 80.0,
                ..Default::default()
            });

            ui.table_next_row();
            ui.table_next_column();
            if let Some(handle) = texture {
                let pool = asset_manager.read_assets().unwrap();
                if let Some(texture) = pool.get(handle) {
                   Image::new(ui.get_texture_id(texture.raw()), [64.0, 64.0])
                   .build(ui);
                }
            }
            ui.table_next_column();
            AssetSelector::new(ui, name, texture_extensions)
            .build(texture, asset_manager);
            (other)();
        }
    }

    Ok(())
}
fn material_edit(
    ui: &Ui,
    current: &mut Option<Handle<Material>>,
    asset_manager: &AssetManager,
) -> anyhow::Result<()> {

    if let Some(_table_token) = ui.begin_table_with_sizing(
        "MaterialEditor",
        2,
        TableFlags::SIZING_STRETCH_PROP,
        [0.0, 0.0],
        0.0,
    ) {
        ui.table_setup_column_with(TableColumnSetup {
            name: "",
            init_width_or_weight: 30.0,
            ..Default::default()
        });

        ui.table_setup_column_with(TableColumnSetup {
            name: "",
            init_width_or_weight: 70.0,
            ..Default::default()
        });

        ui.table_next_row();
        ui.table_next_column();
        ui.text("Material");
        ui.table_next_column();
        AssetSelector::new(ui,  "MaterialAsset", hikari::g3d::SUPPORTED_MATERIAL_EXTENSIONS)
        .build(current, asset_manager);
    }
    if let Some(handle) = &current {
        let mut materials = asset_manager.write_assets::<Material>().unwrap();
        let material = materials.get_mut(&handle);
        if let Some(material) = material {
            parameter_edit(ui, "Albedo", &mut material.albedo, || {
                ui.full_width(|| {
                    ui.color_edit4_config("##AlbedoFactor", &mut material.albedo_factor)
                        .picker(true)
                        .build();
                });
            }, asset_manager)?;

            parameter_edit(ui, "Roughness", &mut material.roughness, || {
                ui.full_width(|| {
                    ui.slider(
                        "##RoughnessFactor",
                        0.0,
                        1.0,
                        &mut material.roughness_factor,
                    );
                });
            }, asset_manager)?;

            parameter_edit(ui, "Metallic", &mut material.metallic, || {
                ui.full_width(|| {
                    ui.slider(
                        "##MetallicFactor",
                        0.0,
                        1.0,
                        &mut material.metallic_factor,
                    );
                });
            }, asset_manager)?;

            parameter_edit(ui, "Emissive", &mut material.emissive, || {
                ui.full_width(|| {
                    ui.color_edit3_config("##EmissiveFactor", &mut material.emissive_factor)
                        .picker(true)
                        .build();
                    Drag::new("##EmissiveStrength")
                    .range(0.0, f32::MAX)
                    .speed(0.25)
                    .build(ui, &mut material.emissive_strength);
                });
            }, asset_manager)?;

            parameter_edit(ui, "Normal", &mut material.normal, || {

            }, asset_manager)?;
            // let _table_token = ui.begin_table_with_sizing(
            //     "MaterialEditor",
            //     2,
            //     TableFlags::SIZING_STRETCH_PROP,
            //     [0.0, 0.0],
            //     0.0,
            // );

            // if let Some(_table_token) = _table_token {
            //     ui.table_setup_column_with(TableColumnSetup {
            //         name: "",
            //         init_width_or_weight: 30.0,
            //         ..Default::default()
            //     });

            //     ui.table_setup_column_with(TableColumnSetup {
            //         name: "",
            //         init_width_or_weight: 70.0,
            //         ..Default::default()
            //     });

            //     ui.table_next_row();
            //     ui.table_next_column();
            //     ui.text("Albedo");
            //     ui.table_next_column();
            //     asset_selector::<Texture2D>(
            //         ui,
            //         "##MaterialAssetAlbedo",
            //         &mut material.albedo,
            //         &asset_manager,
            //         &texture_extensions,
            //     );

            //     ui.table_next_row();
            //     ui.table_next_column();
            //     ui.text("AlbedoFactor");
            //     ui.table_next_column();

            //     ui.full_width(|| {
            //         ui.color_edit4_config("##AlbedoFactor", &mut material.albedo_factor)
            //             .picker(true)
            //             .build();
            //     });

            //     ui.table_next_row();
            //     ui.table_next_column();
            //     ui.text("Roughness");
            //     ui.table_next_column();
            //     asset_selector::<Texture2D>(
            //         ui,
            //         "##MaterialAssetRoughness",
            //         &mut material.roughness,
            //         &asset_manager,
            //         &texture_extensions,
            //     );

            //     ui.table_next_row();
            //     ui.table_next_column();
            //     ui.text("RoughnessFactor");
            //     ui.table_next_column();

            //     ui.full_width(|| {
            //         ui.slider(
            //             "##RoughnessFactor",
            //             0.0,
            //             1.0,
            //             &mut material.roughness_factor,
            //         );
            //     });

            //     ui.table_next_row();
            //     ui.table_next_column();
            //     ui.text("Metallic");
            //     ui.table_next_column();
            //     asset_selector::<Texture2D>(
            //         ui,
            //         "##MaterialAssetMetallic",
            //         &mut material.metallic,
            //         &asset_manager,
            //         &texture_extensions,
            //     );

            //     ui.table_next_row();
            //     ui.table_next_column();
            //     ui.text("MetallicFactor");
            //     ui.table_next_column();

            //     ui.full_width(|| {
            //         ui.slider("##MetallicFactor", 0.0, 1.0, &mut material.metallic_factor);
            //     });

            //     ui.table_next_row();
            //     ui.table_next_column();
            //     ui.text("Normal");
            //     ui.table_next_column();
            //     asset_selector::<Texture2D>(
            //         ui,
            //         "##MaterialAssetNormal",
            //         &mut material.normal,
            //         &asset_manager,
            //         &texture_extensions,
            //     );
            // }

            drop(materials);
            ui.new_line();

            if ui.button("Save") {
                asset_manager.save::<Material>(handle)?;
            }

            // ui.color_edit4_config("NormalStrength", &mut material.normal_strength)
            // .picker(true)
            // .build();
        } else {
            ui.text("Not Loaded");
        }
    }

    Ok(())
}
