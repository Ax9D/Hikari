pub mod app;
pub mod assetmanager;
pub mod context;
pub mod plugin;
pub mod primitives;
pub mod utils;

pub use context::Context;

pub use app::App;
pub use assetmanager::AssetManager;
pub use plugin::Plugin;

pub use primitives::Scene;

use crate::rawToStr;

fn print_cpu_info() {
    match cupid::master() {
        Some(info) => match info.brand_string() {
            Some(brand_str) => {
                println!("CPU: {}", brand_str);
            }
            None => {
                println!("Couldn't get CPU vendor string");
            }
        },
        None => {
            println!("Couldn't get CPU info");
        }
    }
}

pub fn setup_logging() {
    let colors_line = fern::colors::ColoredLevelConfig::new()
        .error(fern::colors::Color::Red)
        .warn(fern::colors::Color::Yellow)
        .info(fern::colors::Color::Green)
        .debug(fern::colors::Color::BrightBlue)
        .trace(fern::colors::Color::BrightBlack);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{bold}{white}[{}]{reset} {}{s_reset} {}\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors_line.color(record.level()),
                message,
                bold = crossterm::style::Attribute::Bold,
                white = crossterm::style::SetForegroundColor(crossterm::style::Color::White),
                reset = crossterm::style::ResetColor,
                s_reset = crossterm::style::Attribute::Reset
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log").unwrap())
        .apply()
        .unwrap();
}
pub fn init() {
    setup_logging();
    println!("{bold}", bold = crossterm::style::Attribute::Bold);
    print_cpu_info();
    //printGPUInfo();
    println!("{reset}", reset = crossterm::style::Attribute::Reset);
}

pub fn update(ctx: &mut crate::Context) {}
