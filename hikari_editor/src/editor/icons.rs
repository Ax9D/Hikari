use hikari::imgui::{FontSource, FontConfig, FontGlyphRanges};

pub const GIZMO_TRANSLATE: &str = "\u{f0597}";
pub const GIZMO_ROTATE: &str = "\u{f0598}";
pub const GIZMO_SCALE: &str = "\u{f0599}";
pub const GIZMO_LOCAL: &str = "\u{f059a}";
pub const GIZMO_WORLD: &str = "\u{f059b}";
pub const MOUSE_SELECT: &str = "\u{f059c}";

pub fn icon_ttf(data: &[u8], hidpi_factor: f32) ->FontSource {
    FontSource::TtfData {
        data,
        size_pixels: 13.0 * hidpi_factor * 1.5,
        config: Some(FontConfig {
            glyph_ranges: FontGlyphRanges::from_slice(&[0xf0597, 0xf059c, 0]),
            ..Default::default()
        }),
    }
}
