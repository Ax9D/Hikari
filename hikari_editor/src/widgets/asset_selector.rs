use std::path::Path;
use std::path::PathBuf;

use hikari::asset::{Asset, AssetManager, Handle};
use hikari::imgui::*;

#[allow(unused)]
struct FancyPopupToken<'a>(&'a Ui);
impl<'a> Drop for FancyPopupToken<'a> {
    fn drop(&mut self) {
        unsafe {
            sys::igEndPopup();
        }
    }
}
fn fancy_popup(ui: &Ui, str_id: impl AsRef<str>, flags: WindowFlags) -> Option<FancyPopupToken> {
    let render = unsafe { sys::igBeginPopup(ui.scratch_txt(str_id), flags.bits() as i32) };

    if render {
        Some(FancyPopupToken(ui))
    } else {
        None
    }
}

fn handle_to_path<T: Asset>(
    handle: &Option<Handle<T>>,
    asset_manager: &AssetManager,
) -> Option<PathBuf> {
    let asset_db = asset_manager.asset_db().read();

    if let Some(handle) = handle {
        let path_buf = asset_db
            .handle_to_path(&handle.clone_erased_as_weak())
            .unwrap();

        Some(path_buf)
    } else {
        None
    }
}
fn path_to_display_string(path: &Option<PathBuf>) -> &str {
    match path {
        Some(path) => {
            let filename = path.file_name().expect("Couldn't find filename");
            filename.to_str().expect("Couldn't convert to str")
        }
        None => "None",
    }
}
pub struct AssetSelector<'ui, L, const N: usize> {
    ui: &'ui Ui,
    name: L,
    extensions: [&'static str; N],
}
impl<'ui, L: AsRef<str>, const N: usize> AssetSelector<'ui, L, N> {
    pub fn new(ui: &'ui Ui, name: L, extensions: [&'static str; N]) -> Self {
        Self {
            ui,
            name,
            extensions,
        }
    }
    pub fn build<T: Asset>(
        self,
        return_handle: &mut Option<Handle<T>>,
        asset_manager: &AssetManager,
    ) -> bool {
        hikari::dev::profile_function!();

        let ui = self.ui;
        let name = self.name.as_ref();
        let extensions = &self.extensions;

        assert!(!extensions.is_empty());

        let current_path = handle_to_path(return_handle, asset_manager);

        let display_path = path_to_display_string(&current_path);

        let _id = ui.push_id(name);

        let path_selector =
            ui.button_with_size(display_path, [ui.content_region_avail()[0] - 20.0, 0.0]);

        if path_selector {
            ui.open_popup("popup");
        }
        unsafe {
            sys::igSetNextWindowPos(
                sys::ImVec2::new(ui.item_rect_min()[0], ui.item_rect_max()[1]),
                Condition::Always as sys::ImGuiCond,
                sys::ImVec2::zero(),
            );
            sys::igSetNextWindowSize(
                sys::ImVec2::new(ui.item_rect_size()[0], 150.0),
                Condition::Always as sys::ImGuiCond,
            );
        }

        let mut changed = false;
        let mut clear = false;

        if return_handle.is_some() {
            ui.same_line_with_spacing(0.0, 2.0);
            clear = ui.button("x");
        }

        if clear {
            *return_handle = None;
        }

        let window_flags =
            WindowFlags::NO_TITLE_BAR | WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE;

        //https://github.com/ocornut/imgui/issues/718
        if let Some(_popup_token) = fancy_popup(ui, "popup", window_flags) {
            //https://github.com/ocornut/imgui/issues/4461
            if ui.is_window_appearing() {
                unsafe {
                    sys::igBringWindowToDisplayFront(sys::igGetCurrentWindow());
                };
            }

            let asset_db = asset_manager.asset_db();
            let asset_db_read = asset_db.read();

            let mut storage = ui.storage();
            let id = ui.new_id_str("AssetSelectorPathBuffer");
            let search_buffer = storage.get_or_insert_with(id, || String::new());

            {
                let full_width = ui.content_region_max()[0];
                let _width_token = ui.push_item_width(full_width);

                ui.input_text("##AssetPathSearch", search_buffer)
                    .hint("Search")
                    .enter_returns_true(true)
                    .build();
            };

            let filtered_records = asset_db_read.records().iter().filter(|record| {
                if let Some(extension) = record.path.extension() {
                    extensions
                        .iter()
                        .find(|&&check| check == extension)
                        .is_some()
                } else {
                    false
                }
            });

            for record in filtered_records {
                let path_str = record.path.as_os_str().to_str().unwrap();
                if path_str
                    .to_lowercase()
                    .contains(&search_buffer.to_lowercase())
                {
                    if ui.selectable(path_str) {
                        unsafe {
                            sys::igClearActiveID();
                            *search_buffer = String::from(path_str);
                            changed = true;
                        }
                    }
                }
            }

            if changed {
                let path = Path::new(&*search_buffer);
                let existing_handle = asset_db_read.path_to_handle(path);

                let handle = if let Some(existing_handle) = existing_handle {
                    existing_handle.clone().into_typed::<T>()
                } else {
                    drop(asset_db_read);
                    asset_manager.load(path, None, false).ok()
                };

                *return_handle = handle;
                search_buffer.clear();
            }

            // if enter_pressed || (!is_input_text_active && !ui.is_window_focused()) {
            //     ui.close_current_popup();
            // }
        }

        changed | clear
    }
}
