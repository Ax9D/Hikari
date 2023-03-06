#![allow(dead_code)]
use std::{collections::VecDeque, ops::Index};

use chrono::Utc;
use fern::colors::{Color, ColoredLevelConfig};
use flume::{self, Receiver, Sender};
use hikari::imgui::{self, ImColor32};
use hikari::imgui::{TableColumnSetup, TableFlags};
use hikari_editor::EngineState;

use super::{Editor, EditorWindow};

pub struct RollingBuffer<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T> RollingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    pub fn push(&mut self, data: T) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }

        if self.capacity != 0 {
            self.buffer.push_back(data);
        }
    }
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    pub fn resize(&mut self, capacity: usize) {
        if self.len() > capacity {
            let extra = self.len() - capacity;
            for _ in 0..extra {
                self.buffer.pop_front();
            }
        }

        self.capacity = capacity;
    }
    pub fn clear(&mut self) {
        self.buffer.clear()
    }
    pub fn iter(&self) -> Iter<T> {
        Iter {
            iter: self.buffer.iter(),
        }
    }
}
impl<T> Index<usize> for RollingBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.buffer.index(index)
    }
}

#[derive(Debug)]
pub struct Line {
    pub message: String,
    pub log_level: log::Level,
    pub timestamp: chrono::DateTime<Utc>,
}
pub struct LogListener {
    buffer: RollingBuffer<Line>,

    sender: Sender<Line>,
    recv: Receiver<Line>,
}
impl LogListener {
    pub fn new(capacity: usize) -> Self {
        let (sender, recv) = flume::unbounded();

        Self {
            buffer: RollingBuffer::new(capacity),
            sender,
            recv,
        }
    }
    pub fn sender(&self) -> &Sender<Line> {
        &self.sender
    }
    fn listen(&mut self) {
        hikari::dev::profile_function!();
        for line in self.recv.try_iter() {
            self.buffer.push(line);
        }
    }
    pub fn lines(&mut self) -> Lines {
        self.listen();

        self.buffer.iter()
    }
    pub fn buffer(&mut self) -> &RollingBuffer<Line> {
        self.listen();

        &self.buffer
    }
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }
    pub fn resize(&mut self, capacity: usize) {
        self.buffer.resize(capacity);
    }
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl<'a> ToString for Lines<'a> {
    fn to_string(&self) -> String {
        let lines = self.clone();
        let mut lines_string = String::with_capacity(lines.size_hint().0 * 100);

        for line in lines {
            let line_string = format!(
                "{} {} {}\n",
                line.timestamp.to_string(),
                line.log_level.to_string(),
                line.message.to_string()
            );
            lines_string.push_str(&line_string);
        }

        lines_string
    }
}

impl<'a> Clone for Lines<'a> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

pub type Lines<'a> = Iter<'a, Line>;

pub struct Iter<'a, T> {
    iter: std::collections::vec_deque::Iter<'a, T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub fn init() -> Result<LogListener, fern::InitError> {
    let mut colors = ColoredLevelConfig::default();
    colors.debug = Color::BrightMagenta;
    colors.info = Color::BrightBlue;
    colors.trace = Color::BrightGreen;

    let log_listener = LogListener::new(1000);
    let sender = log_listener.sender().clone();
    fern::Dispatch::new()
        .format(move |out, message, record| {
            let timestamp = chrono::Utc::now();

            out.finish(format_args!(
                "{}{} {}",
                timestamp.format("[%Y-%m-%d][%H:%M:%S]"),
                colors.color(record.level()),
                message
            ));
            let _result = sender.send(Line {
                message: format!("{}", message),
                log_level: record.level(),
                timestamp,
            });
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::Output::stdout("\n"))
        .chain(fern::log_file("hikari.log").unwrap())
        .apply()?;

    Ok(log_listener)
}

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
