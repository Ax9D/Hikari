use hikari::imgui::{self, ImColor32};
use hikari::imgui::{TableColumnSetup, TableFlags};
use hikari_editor::EngineState;

use crate::editor::logging::*;

use super::{Editor, EditorWindow};

pub struct Logging {
    log_listener: LogListener,
    filter: usize,
    search: String,
}
impl Logging {
    pub fn new(log_listener: LogListener) -> Self {
        Self {
            log_listener,
            filter: 0,
            search: String::new(),
        }
    }
}

fn draw_line(ui: &imgui::Ui, line: &Line) {
    ui.table_next_row();
    hikari::dev::profile_scope!("Draw Lines");
    let color = match line.log_level {
        log::Level::Error => ImColor32::from_rgb(255, 10, 0),
        log::Level::Warn => ImColor32::from_rgb(212, 103, 8),
        log::Level::Info => ImColor32::from_rgb(61, 174, 233),
        log::Level::Debug => ImColor32::from_rgb(142, 68, 173),
        log::Level::Trace => ImColor32::from_rgb(29, 208, 147),
    };

    ui.table_next_column();
    ui.text(
        line.timestamp
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
    );
    ui.table_next_column();
    ui.text_colored(color.to_rgba_f32s(), line.log_level.as_str());
    ui.table_next_column();
    ui.text(&line.message);
    //println!("{} {}", ui.scroll_y(), ui.scroll_max_y());
}
impl EditorWindow for Logging {
    fn draw(ui: &imgui::Ui, editor: &mut Editor, _state: EngineState) -> anyhow::Result<()> {
        //log::debug!("Ayy Lmao");
        ui.window("Engine Log")
            .size([950.0, 200.0], imgui::Condition::FirstUseEver)
            .flags(imgui::WindowFlags::HORIZONTAL_SCROLLBAR)
            .resizable(true)
            .build(|| {
                let logging = &mut editor.logging;
                hikari::dev::profile_scope!("Engine Log");
                let mut nlines = logging.log_listener.capacity() as i32;
                {
                    let _width = ui.push_item_width(300.0);
                    ui.input_int("Buffer Capacity", &mut nlines).build();
                }
                nlines = nlines.max(0);
                if nlines as usize != logging.log_listener.capacity() {
                    logging.log_listener.resize(nlines as usize);
                }
                ui.same_line();

                if ui.button("Clear") {
                    logging.log_listener.clear();
                }
                ui.same_line();
                if ui.button("Copy All to Clipboard") {
                    let lines = logging.log_listener.lines();
                    ui.set_clipboard_text(lines.to_string());
                }
                {
                    let _token = ui.push_item_width(100.0);
                    ui.combo(
                        "Filter",
                        &mut logging.filter,
                        &[
                            None,
                            Some(log::Level::Error),
                            Some(log::Level::Warn),
                            Some(log::Level::Info),
                            Some(log::Level::Debug),
                            Some(log::Level::Trace),
                        ],
                        |level| {
                            std::borrow::Cow::Borrowed(
                                level.map(|level| level.as_str()).unwrap_or("All"),
                            )
                        },
                    );
                }

                {
                    ui.same_line();
                    let _token = ui.push_item_width(500.0);
                    ui.input_text("##Search", &mut logging.search)
                        .hint("Search")
                        .build();
                }
                if let Some(_token) = ui.begin_table_header_with_flags(
                    "LoggingTable",
                    [
                        TableColumnSetup {
                            name: "Timestamp",
                            init_width_or_weight: 20.0,
                            ..Default::default()
                        },
                        TableColumnSetup {
                            name: "Level",
                            init_width_or_weight: 10.0,
                            ..Default::default()
                        },
                        TableColumnSetup {
                            name: "Message",
                            init_width_or_weight: 80.0,
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
                    let lines = logging.log_listener.buffer();

                    if logging.filter == 0 && logging.search.is_empty() {
                        //Special case to speed up normal log viewing
                        let clipper = imgui::ListClipper::new(lines.len() as i32);
                        let mut clipper = clipper.begin(ui);
                        while clipper.step() {
                            for line_ix in clipper.display_start()..clipper.display_end() {
                                draw_line(ui, &lines[line_ix as usize]);
                            }
                        }
                    } else {
                        let filtered = lines
                            .iter()
                            .filter(|line| {
                                if logging.filter == 0 {
                                    return true;
                                }
                                line.log_level as usize == logging.filter
                            })
                            .filter(|line| {
                                line.message
                                    .to_lowercase()
                                    .contains(&logging.search.to_lowercase())
                            });

                        let filtered: Vec<_> = filtered.collect();
                        let clipper = imgui::ListClipper::new(filtered.len() as i32);

                        let mut clipper = clipper.begin(ui);
                        while clipper.step() {
                            for line_ix in clipper.display_start()..clipper.display_end() {
                                draw_line(ui, &filtered[line_ix as usize]);
                            }
                        }
                    }

                    if f32::abs(ui.scroll_y() - ui.scroll_max_y()) <= 1.0 {
                        ui.set_scroll_here_y_with_ratio(1.0);
                    }
                }
            });
        Ok(())
    }
}
