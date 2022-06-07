use std::path::Path;

pub fn load_from_file_with_format(
    path: &Path,
    format: image::ImageFormat,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let data = std::fs::read(path)?;
    Ok(load_from_data(&data, format)?)
}
// pub fn format_from_path(path: &Path) -> Result<image::ImageFormat, image::ImageError>{
//     image::ImageFormat::from_path(path)
// }
pub fn load_from_data(
    data: &[u8],
    format: image::ImageFormat,
) -> Result<(Vec<u8>, u32, u32), image::ImageError> {
    let image = image::load_from_memory_with_format(&data, format)?;
    let width = image.width();
    let height = image.height();
    Ok((image.into_rgba8().to_vec(), width, height))
}
pub fn load_from_file(path: impl AsRef<Path>) -> Result<(Vec<u8>, u32, u32), image::ImageError> {
    let image = image::open(path)?;
    let width = image.width();
    let height = image.height();
    Ok((image.into_rgba8().to_vec(), width, height))
}
