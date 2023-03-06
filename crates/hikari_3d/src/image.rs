use std::{io::BufRead, path::Path};

use image::{EncodableLayout, ImageResult};

pub fn open_rgba8(path: impl AsRef<Path>) -> ImageResult<(Vec<u8>, u32, u32)> {
    let dyn_image = image::open(path)?;

    let image = dyn_image.to_rgba8();
    let (width, height) = image.dimensions();

    Ok((image.to_vec(), width, height))
}
pub fn open_rgba32f(path: impl AsRef<Path>) -> ImageResult<(Vec<u8>, u32, u32)> {
    let dyn_image = image::open(path)?;

    let image = dyn_image.to_rgba32f();
    let (width, height) = image.dimensions();

    Ok((image.as_bytes().to_owned(), width, height))
}
pub fn open_hdr(stream: impl BufRead) -> ImageResult<(Vec<f32>, u32, u32)> {
    let decoder = ::image::codecs::hdr::HdrDecoder::new(stream)?;

    //let image = image::load(ctx.reader(), format)?;
    let width = decoder.metadata().width;
    let height = decoder.metadata().height;

    let image = decoder.read_image_hdr()?;

    //let image = image.to_rgba32f();
    let mut data = Vec::with_capacity(image.len() * 4);

    for ::image::Rgb(p) in image.iter() {
        data.extend_from_slice(p);
        data.push(0.0);
    }

    Ok((data, width, height))
}
