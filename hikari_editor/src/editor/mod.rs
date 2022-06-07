use clipboard::ClipboardProvider;
use hikari::render::*;
use crate::imgui;

use self::{
    content_browser::ContentBrowser,
    logging::{LogListener, Logging}, tools::Tools,
};
pub mod logging;

mod content_browser;
//mod utils;
mod tools;

struct Clipboard(clipboard::ClipboardContext);
impl Clipboard {
    pub fn new() -> Option<Self> {
        clipboard::ClipboardContext::new()
            .ok()
            .map(|context| Self(context))
    }
}
impl imgui::ClipboardBackend for Clipboard {
    fn get(&mut self) -> Option<String> {
        self.0.get_contents().ok().map(|text| text.into())
    }

    fn set(&mut self, text: &str) {
        let _ = self.0.set_contents(text.to_owned());
    }
}
pub struct EditorConfig {
    pub log_listener: LogListener,
    pub hidpi_factor: f32,
}
pub struct Editor {
    content_browser: ContentBrowser,
    logging: Logging,
    tools: Tools,
    show_demo: bool,
}

impl Editor {
    fn set_dark_theme(ctx: &mut imgui::Context) {
        ctx.style_mut().use_dark_colors();
        let style = ctx.style_mut();
        style.colors[imgui::StyleColor::Text as usize] = [1.00, 1.00, 1.00, 1.00];
        style.colors[imgui::StyleColor::TextDisabled as usize] = [0.50, 0.50, 0.50, 1.00];
        style.colors[imgui::StyleColor::WindowBg as usize] = [0.13, 0.14, 0.15, 1.00];
        style.colors[imgui::StyleColor::ChildBg as usize] = [0.13, 0.14, 0.15, 1.00];
        style.colors[imgui::StyleColor::PopupBg as usize] = [0.13, 0.14, 0.15, 1.00];
        style.colors[imgui::StyleColor::Border as usize] = [0.43, 0.43, 0.50, 0.50];
        style.colors[imgui::StyleColor::BorderShadow as usize] = [0.00, 0.00, 0.00, 0.00];
        style.colors[imgui::StyleColor::FrameBg as usize] = [0.25, 0.25, 0.25, 1.00];
        style.colors[imgui::StyleColor::FrameBgHovered as usize] = [0.38, 0.38, 0.38, 1.00];
        style.colors[imgui::StyleColor::FrameBgActive as usize] = [0.67, 0.67, 0.67, 0.39];
        style.colors[imgui::StyleColor::TitleBg as usize] = [0.08, 0.08, 0.09, 1.00];
        style.colors[imgui::StyleColor::TitleBgActive as usize] = [0.08, 0.08, 0.09, 1.00];
        style.colors[imgui::StyleColor::TitleBgCollapsed as usize] = [0.00, 0.00, 0.00, 0.51];
        style.colors[imgui::StyleColor::MenuBarBg as usize] = [0.14, 0.14, 0.14, 1.00];
        style.colors[imgui::StyleColor::ScrollbarBg as usize] = [0.260, 0.260, 0.260, 0.285];
        style.colors[imgui::StyleColor::ScrollbarGrab as usize] = [0.31, 0.31, 0.31, 1.00];
        style.colors[imgui::StyleColor::ScrollbarGrabHovered as usize] = [0.41, 0.41, 0.41, 1.00];
        style.colors[imgui::StyleColor::ScrollbarGrabActive as usize] = [0.51, 0.51, 0.51, 1.00];
        style.colors[imgui::StyleColor::CheckMark as usize] = [0.11, 0.64, 0.92, 1.00];
        style.colors[imgui::StyleColor::SliderGrab as usize] = [0.11, 0.64, 0.92, 1.00];
        style.colors[imgui::StyleColor::SliderGrabActive as usize] = [0.08, 0.50, 0.72, 1.00];
        style.colors[imgui::StyleColor::Button as usize] = [0.25, 0.25, 0.25, 1.00];
        style.colors[imgui::StyleColor::ButtonHovered as usize] = [0.38, 0.38, 0.38, 1.00];
        style.colors[imgui::StyleColor::ButtonActive as usize] = [0.67, 0.67, 0.67, 0.39];
        style.colors[imgui::StyleColor::Header as usize] = [0.22, 0.22, 0.22, 1.00];
        style.colors[imgui::StyleColor::HeaderHovered as usize] = [0.25, 0.25, 0.25, 1.00];
        style.colors[imgui::StyleColor::HeaderActive as usize] = [0.67, 0.67, 0.67, 0.39];
        style.colors[imgui::StyleColor::Separator as usize] =
            style.colors[imgui::StyleColor::Border as usize];
        style.colors[imgui::StyleColor::SeparatorHovered as usize] = [0.41, 0.42, 0.44, 1.00];
        style.colors[imgui::StyleColor::SeparatorActive as usize] = [0.26, 0.59, 0.98, 0.95];
        style.colors[imgui::StyleColor::ResizeGrip as usize] = [0.00, 0.00, 0.00, 0.00];
        style.colors[imgui::StyleColor::ResizeGripHovered as usize] = [0.29, 0.30, 0.31, 0.67];
        style.colors[imgui::StyleColor::ResizeGripActive as usize] = [0.26, 0.59, 0.98, 0.95];
        style.colors[imgui::StyleColor::Tab as usize] = [0.08, 0.08, 0.09, 0.83];
        style.colors[imgui::StyleColor::TabHovered as usize] = [0.33, 0.34, 0.36, 0.83];
        style.colors[imgui::StyleColor::TabActive as usize] = [0.23, 0.23, 0.24, 1.00];
        style.colors[imgui::StyleColor::TabUnfocused as usize] = [0.08, 0.08, 0.09, 1.00];
        style.colors[imgui::StyleColor::TabUnfocusedActive as usize] = [0.13, 0.14, 0.15, 1.00];
        style.colors[imgui::StyleColor::DockingPreview as usize] = [0.26, 0.59, 0.98, 0.70];
        style.colors[imgui::StyleColor::DockingEmptyBg as usize] = [0.20, 0.20, 0.20, 1.00];
        style.colors[imgui::StyleColor::PlotLines as usize] = [0.61, 0.61, 0.61, 1.00];
        style.colors[imgui::StyleColor::PlotLinesHovered as usize] = [1.00, 0.43, 0.35, 1.00];
        style.colors[imgui::StyleColor::PlotHistogram as usize] = [0.90, 0.70, 0.00, 1.00];
        style.colors[imgui::StyleColor::PlotHistogramHovered as usize] = [1.00, 0.60, 0.00, 1.00];
        style.colors[imgui::StyleColor::TextSelectedBg as usize] = [0.26, 0.59, 0.98, 0.35];
        style.colors[imgui::StyleColor::DragDropTarget as usize] = [0.11, 0.64, 0.92, 1.00];
        style.colors[imgui::StyleColor::NavHighlight as usize] = [0.26, 0.59, 0.98, 1.00];
        style.colors[imgui::StyleColor::NavWindowingHighlight as usize] = [1.00, 1.00, 1.00, 0.70];
        style.colors[imgui::StyleColor::NavWindowingDimBg as usize] = [0.80, 0.80, 0.80, 0.20];
        style.colors[imgui::StyleColor::ModalWindowDimBg as usize] = [0.80, 0.0, 0.8, 0.35];
        style.colors[imgui::StyleColor::CheckMark as usize] = [0.71, 0.71, 0.71, 1.00];
        style.colors[imgui::StyleColor::SliderGrab as usize] = [0.71, 0.71, 0.71, 1.00];
        style.colors[imgui::StyleColor::DockingPreview as usize] = [0.36, 0.37, 0.38, 0.70];
    }
    pub fn new(ctx: &mut imgui::Context, config: EditorConfig) -> Self {
        ctx.style_mut().tab_rounding = 0.0;
        ctx.style_mut().frame_rounding = 2.0;
        ctx.io_mut().config_flags = imgui::ConfigFlags::DOCKING_ENABLE;

        ctx.fonts().add_font(&[imgui::FontSource::TtfData {
            data: include_bytes!("../../../assets/fonts/Roboto/Roboto-Regular.ttf"),
            size_pixels: 13.0 * config.hidpi_factor * 1.5,
            config: None,
        }]);

        if let Some(clipboard) = Clipboard::new() {
            ctx.set_clipboard_backend(clipboard);
        } else {
            log::error!("Failed to init clipboard");
        }
        Self::set_dark_theme(ctx);
        Self {
            logging: Logging::new(config.log_listener),
            tools: Tools::new(),
            show_demo: false,
            content_browser: ContentBrowser::new(),
        }
    }
    pub fn run(&mut self, ui: &imgui::Ui) {
        ui.window("Main")
            .flags(
                imgui::WindowFlags::NO_DECORATION
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::MENU_BAR
                    | imgui::WindowFlags::NO_DOCKING
                    | imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS
                    | imgui::WindowFlags::NO_NAV_FOCUS,
            )
            .position([0.0, 0.0], imgui::Condition::Always)
            .size(ui.io().display_size, imgui::Condition::Always)
            .build(|| {
                ui.dockspace("Dockspace");

                ui.menu_bar(|| {
                    ui.menu("File", || {
                        ui.menu_item_config("Open")
                            .enabled(false)
                            .shortcut("Ctrl + O")
                            .build();
                        ui.menu_item_config("Save")
                            .enabled(false)
                            .shortcut("Ctrl + S")
                            .build();
                        ui.menu_item_config("Save As").enabled(false).build();
                    });

                    ui.menu("Edit", || {
                        ui.menu_item_config("Preferences").enabled(false).build();
                    });
                    ui.menu("Tools", || {
                        if ui.menu_item("Start Tracy") {
                            let path = std::path::Path::new("./tools/");
                            #[cfg(target_os = "windows")]
                            let tracy_exe = "Tracy.exe";
                            #[cfg(target_os = "linux")]
                            let tracy_exe = "tracy";

                            let result = std::process::Command::new(path.join(tracy_exe))
                            .spawn();

                            if let Err(err) = result {
                                log::error!("Failed to spawn tracy: {}", err);
                            }
                        }
                    });
                    ui.menu("Help", || {
                        self.show_demo = ui.menu_item_config("Demo Window").build();
                    });
                });
            });

        self.outliner(ui);
        self.properties(ui);
        content_browser::draw(ui, self);
        self.viewport(ui);
        logging::draw(ui, self);

        if self.show_demo {
            ui.show_demo_window(&mut self.show_demo);
        }
    }

    fn outliner(&mut self, ui: &imgui::Ui) {
        ui.window("Outliner")
            .size([300.0, 400.0], imgui::Condition::Once)
            .resizable(true)
            .build(|| {});
    }
    fn properties(&mut self, ui: &imgui::Ui) {
        ui.window("Properties")
            .size([300.0, 400.0], imgui::Condition::Once)
            .resizable(true)
            .build(|| {});
    }
    fn viewport(&mut self, ui: &imgui::Ui) {
        ui.window("Viewport")
            .size([950.0, 200.0], imgui::Condition::Once)
            .resizable(true)
            .build(|| {});
    }

    pub fn handle_exit(&mut self) {
        log::info!("Editor Exiting");
    }
}
