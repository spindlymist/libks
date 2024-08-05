use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum DrawError {
    #[error("Failed to load image from {path:?}")]
    Image {
        source: image::ImageError,
        path: PathBuf,
    }
}
