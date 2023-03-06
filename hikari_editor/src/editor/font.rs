use hikari::imgui::*;

use super::{icons, EditorConfig};

pub fn load_fonts(ctx: &mut Context, config: &EditorConfig) {
    let font_path = hikari::utils::engine_dir().join("data/assets/fonts/Roboto/Roboto-Regular.ttf");
    let icon_path = hikari::utils::engine_dir().join("data/assets/fonts/icons/icons.ttf");
    let font_data = std::fs::read(font_path).expect("Failed to load font");
    let icon_data = std::fs::read(icon_path).expect("Failed to load icons");

    let font = [
        FontSource::TtfData {
            data: &font_data,
            size_pixels: 11.0 * config.hidpi_factor * 1.5,
            config: Some(FontConfig {
                glyph_ranges: FontGlyphRanges::japanese(),
                ..Default::default()
            }),
        },
        icons::icon_ttf(&icon_data, config.hidpi_factor),
    ];

    ctx.fonts().add_font(&font);
}
