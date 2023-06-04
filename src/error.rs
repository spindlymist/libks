#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An IO error occurred: `{source}`")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("Utf-8 sequence was invalid.")]
    FromUtf8 {
        #[from]
        source: std::string::FromUtf8Error,
    },
    #[error("Utf-8 sequence was invalid.")]
    Utf8 {
        #[from]
        source: std::str::Utf8Error,
    },
    #[error("Unable to parse number from string.")]
    ParseIntError {
        #[from]
        source: std::num::ParseIntError,
    },
    #[error(transparent)]
    KnyttBin(#[from] crate::KnyttBinError),
    #[error(transparent)]
    MapBin(#[from] crate::MapBinError),
    #[error(transparent)]
    WorldIni(#[from] crate::WorldIniError),
}

pub type Result<T> = core::result::Result<T, Error>;
