use std::path::Path;
use std::path::PathBuf;

const VERSION_FILENAME: &str = "HIKARI_VERSION";

#[derive(Debug)]
pub struct Config {
    pub engine_path: PathBuf,
    pub editor_path: PathBuf,
}

#[cfg(target_os = "linux")]
fn get_install_path() -> PathBuf {
    Path::new("/opt/Hikari").to_owned()
}
#[cfg(target_os = "windows")]
fn get_install_path() -> PathBuf {
    //TODO: Do a registry lookup for the actual install location
    Path::new("C:/Hikari").to_owned()
}
impl Config {
    fn find_engine_path() -> Result<PathBuf, anyhow::Error> {
        let install_path = get_install_path();

        let engine_path = if install_path.exists() {
            install_path
        } else {
            std::env::current_dir()?
        };

        let cli_version = env!("CARGO_PKG_VERSION");

        let version_filepath = engine_path.join(VERSION_FILENAME);

        if !version_filepath.exists() {
            return Err(anyhow::anyhow!("Couldn't locate Hikari. Is it installed?"));
        }

        let version = std::fs::read_to_string(version_filepath)?;

        if version.trim() != cli_version {
            return Err(anyhow::anyhow!(
                "Version mismatch! CLI version: {}, Engine version: {}",
                cli_version,
                version
            ));
        }

        Ok(engine_path)
    }
    fn find_editor_path(engine_path: &Path) -> Result<PathBuf, anyhow::Error> {
        #[cfg(target_os = "linux")]
        let editor_binary = "hikari_editor";
        #[cfg(target_os = "windows")]
        let editor_binary = "hikari_editor.exe";

        let install_editor_path = engine_path.join(Path::new(editor_binary));

        // For when we are running in a dev environment
        let exec_editor_path = std::env::current_exe()?
            .parent()
            .expect("No parent???")
            .join(Path::new(editor_binary));

        let editor_path = if install_editor_path.is_file() {
            Ok(install_editor_path)
        } else if exec_editor_path.is_file() {
            Ok(exec_editor_path)
        } else {
            Err(anyhow::anyhow!("Editor binary not found!"))
        }?;

        Ok(editor_path)
    }
    pub fn new() -> Result<Config, anyhow::Error> {
        let engine_path = Self::find_engine_path()?;
        let editor_path = Self::find_editor_path(&engine_path)?;

        Ok(Config {
            engine_path,
            editor_path,
        })
    }
}
