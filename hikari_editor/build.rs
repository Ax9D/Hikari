use fs_extra::dir::CopyOptions;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

fn set_git_hash() -> Result<(), Box<dyn Error>> {
    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output()?;
    let git_hash = String::from_utf8(output.stdout)?;
    println!("cargo:rustc-rerun-if-changed=.git/HEAD");
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    Ok(())
}

fn copy_to_target(folder: &str, root_dir: &Path, target_dir: &Path) -> fs_extra::error::Result<()> {
    let folder = root_dir.join(folder);
    fs_extra::copy_items(
        &[folder],
        target_dir,
        &CopyOptions {
            overwrite: true,
            ..Default::default()
        },
    )?;
    Ok(())
}
fn profile_target_dir() -> PathBuf {
    //Not using PROFILE because it doesn't return custom profile names
    let out_dir = env::var("OUT_DIR").unwrap();
    // This is out_dir:
    // Whatever\\dist\\build\\hikari_editor-abcd1234\\out
    let out_dir = Path::new(&out_dir);

    let mut ancestors = out_dir.ancestors();
    // Whatever\\dist\\build\\hikari_editor-abcd1234
    ancestors.next();
    // Whatever\\dist\\build
    ancestors.next();
    // Whatever\\dist
    let profile_path = ancestors.next().unwrap();

    profile_path.to_owned()
}
fn copy_engine_assets() {
    let editor_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let editor_dir = Path::new(&editor_dir);
    let target_dir = &profile_target_dir();
    let root = editor_dir.parent().unwrap();

    // Re-runs script if any files in res are changed
    println!("cargo:rerun-if-changed=../data*");
    copy_to_target("data", root, target_dir)
        .expect("Could not copy engine data. Is the current directory the root of the repo?");
}
fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    set_git_hash()?;
    copy_engine_assets();

    Ok(())
}
