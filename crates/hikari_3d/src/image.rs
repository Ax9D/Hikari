use std::path::Path;

use image::ImageResult;

pub fn open_rgba8(path: impl AsRef<Path>) -> ImageResult<(Vec<u8>, u32, u32)> {
    let dyn_image = image::open(path)?;

    let image = dyn_image.to_rgba8();
    let (width, height) = image.dimensions();

    Ok((image.to_vec(), width, height))
}
