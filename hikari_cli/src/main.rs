use clap::*;
use simple_logger::SimpleLogger;
use std::path::PathBuf;

mod build;
mod config;
mod new;
mod open;

#[derive(Parser, Debug)]
#[clap(name = "Hikari", version, about = "A CLI for Hikari Engine")]
enum Command {
    /// Create a new Hikari project
    New { path: PathBuf },
    /// Open an existing project
    Open { path: Option<PathBuf> },
    /// Build the game and generate shippable artifacts
    Build {
        /// Build in release mode, with optimizations
        #[clap(long)]
        release: bool,
    },
}
fn run() -> anyhow::Result<()> {
    let cmd = Command::parse();
    let _config = config::Config::new()?;

    match cmd {
        Command::New { path } => {
            new::run(path)
        }
        Command::Open { path } => open::run(path),
        Command::Build { release } => build::run(release),
    }
}
fn main() -> anyhow::Result<()>{
    if SimpleLogger::new().init().is_err() {
        println!("Failed to init logger");
    }

    run()
}
