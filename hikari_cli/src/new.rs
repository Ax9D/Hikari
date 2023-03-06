use std::path::PathBuf;

pub fn run(path: PathBuf) -> anyhow::Result<()> {
    if path.exists() {
        return Err(anyhow::anyhow!(
            "Destination {} already exists",
            path.display()
        ));
    }
    let file_name = path.file_name().unwrap().to_str().ok_or(anyhow::anyhow!(
        "Project name must be a valid unicode string"
    ))?;

    let mut file_name_with_extension = PathBuf::from(&file_name);
    file_name_with_extension.set_extension(hikari_editor::PROJECT_EXTENSION);
    let file_path = path.join(file_name_with_extension);

    std::fs::create_dir(&path)?;

    hikari_editor::Project::new(&file_name)
        .save(&file_path)
        .unwrap();

    println!("Created project: {:?}", file_name);

    Ok(())
}
