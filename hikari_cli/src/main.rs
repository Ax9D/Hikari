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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if SimpleLogger::new().init().is_err() {
        println!("Failed to init logger");
    }

    let cmd = Command::parse();
    let config = config::Config::new()?;

    log::debug!("{:#?}", config);

    println!("{:?}", cmd);

    match cmd {
        Command::New { path } => {
            //new::run(name)
        }
        Command::Open { path } => open::run(path),
        Command::Build { release } => build::run(release),
    }

    Ok(())
}
