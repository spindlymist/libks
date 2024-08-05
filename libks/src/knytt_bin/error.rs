use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum KnyttBinError {
    #[error("An entry began with an unrecognized signature.")]
    UnrecognizedSignature([u8; 2]),
    #[error("An entry had no path specified.")]
    EmptyPath,
    #[error("The path `{0:?}` is not allowed.")]
    IllegalPath(PathBuf),
    #[error("Failed to get name of file or directory `{0:?}`. (hint: is it root or invalid Utf-8?)")]
    BadFileName(PathBuf),
    #[error("The file `{path}` is too large: {size} bytes")]
    OversizedFile {
        path: PathBuf,
        size: usize,
    },
    #[error("The file `{path}` is missing data: {bytes_read}/{file_size} bytes")]
    MissingData {
        path: PathBuf,
        file_size: usize,
        bytes_read: usize,
    },
    #[error("The file or directory `{0}` could not be overwritten.")]
    UnauthorizedOverwrite(PathBuf),
}
