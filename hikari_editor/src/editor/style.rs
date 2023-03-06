use imgui::StyleColor;

use crate::imgui;

pub fn set_dark_theme(ctx: &mut imgui::Context) {
    ctx.style_mut().use_dark_colors();
    let style = ctx.style_mut();
    style.indent_spacing = 0.0;

    style.window_rounding = 6.0;
    style.child_rounding = 6.0;
    style.frame_rounding = 3.0;
    style.popup_rounding = 3.0;
    style.tab_rounding = 4.0;
    style.scrollbar_rounding = 9.0;

    style.window_border_size = 0.0;
    style.child_border_size = 0.0;
    style.popup_border_size = 0.0;

    style.window_title_align = [0.5, 0.5];
    style.button_text_align = [0.5, 0.5];

    style.window_menu_button_position = imgui::Direction::Right;

    let colors = &mut style.colors;

    colors[StyleColor::Text as usize] = [0.78, 0.78, 0.78, 1.0];
    colors[StyleColor::TextDisabled as usize] = [0.44, 0.44, 0.44, 1.0];
    colors[StyleColor::WindowBg as usize] = [0.1, 0.11, 0.13, 1.0];
    colors[StyleColor::ChildBg as usize] = [0.13, 0.14, 0.15, 1.0];
    colors[StyleColor::PopupBg as usize] = [0.13, 0.14, 0.15, 1.0];
    colors[StyleColor::Border as usize] = [0.6, 0.6, 0.6, 0.5];
    colors[StyleColor::BorderShadow as usize] = [0.0, 0.0, 0.0, 0.0];
    colors[StyleColor::FrameBg as usize] = [0.16, 0.18, 0.19, 1.0];
    colors[StyleColor::FrameBgHovered as usize] = [0.3, 0.33, 0.39, 1.0];
    colors[StyleColor::FrameBgActive as usize] = [0.4, 0.41, 0.43, 1.0];
    colors[StyleColor::TitleBg as usize] = [0.18, 0.18, 0.21, 1.0];
    colors[StyleColor::TitleBgActive as usize] = [0.14, 0.14, 0.19, 1.0];
    colors[StyleColor::TitleBgCollapsed as usize] = [0.0, 0.0, 0.0, 0.51];
    colors[StyleColor::MenuBarBg as usize] = [0.14, 0.14, 0.14, 1.0];
    colors[StyleColor::ScrollbarBg as usize] = [0.26, 0.26, 0.26, 0.28];
    colors[StyleColor::ScrollbarGrab as usize] = [0.31, 0.31, 0.31, 1.0];
    colors[StyleColor::ScrollbarGrabHovered as usize] = [0.41, 0.41, 0.41, 1.0];
    colors[StyleColor::ScrollbarGrabActive as usize] = [0.51, 0.51, 0.51, 1.0];
    colors[StyleColor::CheckMark as usize] = [0.71, 0.71, 0.71, 1.0];
    colors[StyleColor::SliderGrab as usize] = [0.71, 0.71, 0.71, 1.0];
    colors[StyleColor::SliderGrabActive as usize] = [0.08, 0.5, 0.72, 1.0];
    colors[StyleColor::Button as usize] = [0.19, 0.19, 0.25, 1.0];
    colors[StyleColor::ButtonHovered as usize] = [0.28, 0.29, 0.38, 1.0];
    colors[StyleColor::ButtonActive as usize] = [0.41, 0.43, 0.56, 1.0];
    colors[StyleColor::Header as usize] = [0.18, 0.18, 0.21, 1.0];
    colors[StyleColor::HeaderHovered as usize] = [0.21, 0.21, 0.26, 1.0];
    colors[StyleColor::HeaderActive as usize] = [0.14, 0.14, 0.19, 1.0];
    colors[StyleColor::Separator as usize] = [0.43, 0.43, 0.5, 0.5];
    colors[StyleColor::SeparatorHovered as usize] = [0.41, 0.42, 0.44, 1.0];
    colors[StyleColor::SeparatorActive as usize] = [0.26, 0.59, 0.98, 0.95];
    colors[StyleColor::ResizeGrip as usize] = [0.0, 0.0, 0.0, 0.0];
    colors[StyleColor::ResizeGripHovered as usize] = [0.29, 0.3, 0.31, 0.67];
    colors[StyleColor::ResizeGripActive as usize] = [0.26, 0.59, 0.98, 0.95];
    colors[StyleColor::Tab as usize] = [0.25, 0.25, 0.25, 0.83];
    colors[StyleColor::TabHovered as usize] = [0.09, 0.48, 0.72, 1.0];
    colors[StyleColor::TabActive as usize] = [0.0, 0.39, 0.64, 1.0];
    colors[StyleColor::TabUnfocused as usize] = [0.08, 0.08, 0.09, 1.0];
    colors[StyleColor::TabUnfocusedActive as usize] = [0.13, 0.14, 0.15, 1.0];
    colors[StyleColor::DockingPreview as usize] = [0.14, 0.88, 0.82, 0.7];
    colors[StyleColor::DockingEmptyBg as usize] = [0.2, 0.2, 0.2, 1.0];
    colors[StyleColor::PlotLines as usize] = [0.61, 0.61, 0.61, 1.0];
    colors[StyleColor::PlotLinesHovered as usize] = [1.0, 0.43, 0.35, 1.0];
    colors[StyleColor::PlotHistogram as usize] = [0.9, 0.7, 0.0, 1.0];
    colors[StyleColor::PlotHistogramHovered as usize] = [1.0, 0.6, 0.0, 1.0];
    colors[StyleColor::TableHeaderBg as usize] = [0.19, 0.19, 0.2, 1.0];
    colors[StyleColor::TableBorderStrong as usize] = [0.31, 0.31, 0.35, 1.0];
    colors[StyleColor::TableBorderLight as usize] = [0.23, 0.23, 0.25, 1.0];
    colors[StyleColor::TableRowBg as usize] = [0.0, 0.0, 0.0, 0.0];
    colors[StyleColor::TableRowBgAlt as usize] = [1.0, 1.0, 1.0, 0.06];
    colors[StyleColor::TextSelectedBg as usize] = [0.26, 0.59, 0.98, 0.35];
    colors[StyleColor::DragDropTarget as usize] = [0.11, 0.64, 0.92, 1.0];
    colors[StyleColor::NavHighlight as usize] = [0.26, 0.59, 0.98, 1.0];
    colors[StyleColor::NavWindowingHighlight as usize] = [1.0, 1.0, 1.0, 0.7];
    colors[StyleColor::NavWindowingDimBg as usize] = [0.8, 0.8, 0.8, 0.2];
    colors[StyleColor::ModalWindowDimBg as usize] = [0.68, 0.68, 0.68, 0.35];
}

pub fn copy_style_to_clipboard_as_rust(ui: &imgui::Ui) {
    let colors = &ui.clone_style().colors;

    let text = format!(
        r"
    colors[StyleColor::Text as usize]                   = {:?};
    colors[StyleColor::TextDisabled as usize]           = {:?};
    colors[StyleColor::WindowBg as usize]               = {:?};
    colors[StyleColor::ChildBg as usize]                = {:?};
    colors[StyleColor::PopupBg as usize]                = {:?};
    colors[StyleColor::Border as usize]                 = {:?};
    colors[StyleColor::BorderShadow as usize]           = {:?};
    colors[StyleColor::FrameBg as usize]                = {:?};
    colors[StyleColor::FrameBgHovered as usize]         = {:?};
    colors[StyleColor::FrameBgActive as usize]          = {:?};
    colors[StyleColor::TitleBg as usize]                = {:?};
    colors[StyleColor::TitleBgActive as usize]          = {:?};
    colors[StyleColor::TitleBgCollapsed as usize]       = {:?};
    colors[StyleColor::MenuBarBg as usize]              = {:?};
    colors[StyleColor::ScrollbarBg as usize]            = {:?};
    colors[StyleColor::ScrollbarGrab as usize]          = {:?};
    colors[StyleColor::ScrollbarGrabHovered as usize]   = {:?};
    colors[StyleColor::ScrollbarGrabActive as usize]    = {:?};
    colors[StyleColor::CheckMark as usize]              = {:?};
    colors[StyleColor::SliderGrab as usize]             = {:?};
    colors[StyleColor::SliderGrabActive as usize]       = {:?};
    colors[StyleColor::Button as usize]                 = {:?};
    colors[StyleColor::ButtonHovered as usize]          = {:?};
    colors[StyleColor::ButtonActive as usize]           = {:?};
    colors[StyleColor::Header as usize]                 = {:?};
    colors[StyleColor::HeaderHovered as usize]          = {:?};
    colors[StyleColor::HeaderActive as usize]           = {:?};
    colors[StyleColor::Separator as usize]              = {:?};
    colors[StyleColor::SeparatorHovered as usize]       = {:?};
    colors[StyleColor::SeparatorActive as usize]        = {:?};
    colors[StyleColor::ResizeGrip as usize]             = {:?};
    colors[StyleColor::ResizeGripHovered as usize]      = {:?};
    colors[StyleColor::ResizeGripActive as usize]       = {:?};
    colors[StyleColor::Tab as usize]                    = {:?};
    colors[StyleColor::TabHovered as usize]             = {:?};
    colors[StyleColor::TabActive as usize]              = {:?};
    colors[StyleColor::TabUnfocused as usize]           = {:?};
    colors[StyleColor::TabUnfocusedActive as usize]     = {:?};
    colors[StyleColor::DockingPreview as usize]         = {:?};
    colors[StyleColor::DockingEmptyBg as usize]         = {:?};
    colors[StyleColor::PlotLines as usize]              = {:?};
    colors[StyleColor::PlotLinesHovered as usize]       = {:?};
    colors[StyleColor::PlotHistogram as usize]          = {:?};
    colors[StyleColor::PlotHistogramHovered as usize]   = {:?};
    colors[StyleColor::TableHeaderBg as usize]          = {:?};
    colors[StyleColor::TableBorderStrong as usize]      = {:?};
    colors[StyleColor::TableBorderLight as usize]       = {:?};
    colors[StyleColor::TableRowBg as usize]             = {:?};
    colors[StyleColor::TableRowBgAlt as usize]          = {:?};
    colors[StyleColor::TextSelectedBg as usize]         = {:?};
    colors[StyleColor::DragDropTarget as usize]         = {:?};
    colors[StyleColor::NavHighlight as usize]           = {:?};
    colors[StyleColor::NavWindowingHighlight as usize]  = {:?};
    colors[StyleColor::NavWindowingDimBg as usize]      = {:?};
    colors[StyleColor::ModalWindowDimBg as usize]       = {:?};
    ",
        colors[StyleColor::Text as usize],
        colors[StyleColor::TextDisabled as usize],
        colors[StyleColor::WindowBg as usize],
        colors[StyleColor::ChildBg as usize],
        colors[StyleColor::PopupBg as usize],
        colors[StyleColor::Border as usize],
        colors[StyleColor::BorderShadow as usize],
        colors[StyleColor::FrameBg as usize],
        colors[StyleColor::FrameBgHovered as usize],
        colors[StyleColor::FrameBgActive as usize],
        colors[StyleColor::TitleBg as usize],
        colors[StyleColor::TitleBgActive as usize],
        colors[StyleColor::TitleBgCollapsed as usize],
        colors[StyleColor::MenuBarBg as usize],
        colors[StyleColor::ScrollbarBg as usize],
        colors[StyleColor::ScrollbarGrab as usize],
        colors[StyleColor::ScrollbarGrabHovered as usize],
        colors[StyleColor::ScrollbarGrabActive as usize],
        colors[StyleColor::CheckMark as usize],
        colors[StyleColor::SliderGrab as usize],
        colors[StyleColor::SliderGrabActive as usize],
        colors[StyleColor::Button as usize],
        colors[StyleColor::ButtonHovered as usize],
        colors[StyleColor::ButtonActive as usize],
        colors[StyleColor::Header as usize],
        colors[StyleColor::HeaderHovered as usize],
        colors[StyleColor::HeaderActive as usize],
        colors[StyleColor::Separator as usize],
        colors[StyleColor::SeparatorHovered as usize],
        colors[StyleColor::SeparatorActive as usize],
        colors[StyleColor::ResizeGrip as usize],
        colors[StyleColor::ResizeGripHovered as usize],
        colors[StyleColor::ResizeGripActive as usize],
        colors[StyleColor::Tab as usize],
        colors[StyleColor::TabHovered as usize],
        colors[StyleColor::TabActive as usize],
        colors[StyleColor::TabUnfocused as usize],
        colors[StyleColor::TabUnfocusedActive as usize],
        colors[StyleColor::DockingPreview as usize],
        colors[StyleColor::DockingEmptyBg as usize],
        colors[StyleColor::PlotLines as usize],
        colors[StyleColor::PlotLinesHovered as usize],
        colors[StyleColor::PlotHistogram as usize],
        colors[StyleColor::PlotHistogramHovered as usize],
        colors[StyleColor::TableHeaderBg as usize],
        colors[StyleColor::TableBorderStrong as usize],
        colors[StyleColor::TableBorderLight as usize],
        colors[StyleColor::TableRowBg as usize],
        colors[StyleColor::TableRowBgAlt as usize],
        colors[StyleColor::TextSelectedBg as usize],
        colors[StyleColor::DragDropTarget as usize],
        colors[StyleColor::NavHighlight as usize],
        colors[StyleColor::NavWindowingHighlight as usize],
        colors[StyleColor::NavWindowingDimBg as usize],
        colors[StyleColor::ModalWindowDimBg as usize]
    );

    ui.set_clipboard_text(text)
}
