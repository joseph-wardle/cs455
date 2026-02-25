use std::path::Path;

use image::{ColorType, ImageFormat};
use thiserror::Error;

use crate::render::RenderImage;

#[derive(Debug, Error)]
pub enum ImageWriteError {
    #[error("failed to write PNG to '{path}': {source}")]
    PngEncode {
        path: String,
        #[source]
        source: image::ImageError,
    },
}

pub fn write_png(path: &Path, image: &RenderImage) -> Result<(), ImageWriteError> {
    image::save_buffer_with_format(
        path,
        image.as_rgb8(),
        image.width(),
        image.height(),
        ColorType::Rgb8,
        ImageFormat::Png,
    )
    .map_err(|source| ImageWriteError::PngEncode {
        path: path.display().to_string(),
        source,
    })
}
