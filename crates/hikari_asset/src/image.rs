use std::path::Path;

use image::GenericImageView;

pub enum ImageFormat {
    Png,
    Jpeg,
}
impl Into<image::ImageFormat> for ImageFormat {
    fn into(self) -> image::ImageFormat {
        match self {
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
        }
    }
}

pub fn load_from_file_with_format(
    path: &Path,
    format: ImageFormat,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let data = std::fs::read(path)?;
    Ok(load_from_data(&data, format)?)
}
pub fn load_from_data(
    data: &[u8],
    format: ImageFormat,
) -> Result<(Vec<u8>, u32, u32), image::ImageError> {
    let format: image::ImageFormat = format.into();
    let image = image::load_from_memory_with_format(&data, format)?;
    let width = image.width();
    let height = image.height();
    Ok((image.into_rgba8().to_vec(), width, height))
}
pub fn load_from_file(path: &Path) -> Result<(Vec<u8>, u32, u32), image::ImageError> {
    let image = image::open(path)?;
    let width = image.width();
    let height = image.height();
    Ok((image.into_rgba8().to_vec(), width, height))
}
