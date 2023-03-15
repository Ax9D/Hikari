use std::{collections::VecDeque, ops::Index};

use chrono::Utc;
use fern::colors::{ColoredLevelConfig, Color};
use flume::{Receiver, Sender};

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
    #[allow(unused)]
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
    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    #[allow(unused)]
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