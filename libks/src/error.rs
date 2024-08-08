use thiserror::Error;

#[derive(Error, Debug)]
pub enum KsError {
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
    #[cfg(feature="image")]
    #[error(transparent)]
    Draw(#[from] crate::DrawError),
    #[error(transparent)]
    ReadString(#[from] crate::io_util::ReadStringError),
}

pub type Result<T> = core::result::Result<T, KsError>;
