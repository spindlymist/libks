use std::backtrace::Backtrace;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An IO error occurred: `{source}`")]
    Io {
        #[from]
        source: std::io::Error,
        backtrace: Backtrace,
    },
    #[error("Utf-8 sequence was invalid.")]
    FromUtf8 {
        #[from]
        source: std::string::FromUtf8Error,
        backtrace: Backtrace,
    },
    #[error(transparent)]
    KnyttBin(#[from] crate::KnyttBinError),
}

pub type Result<T> = core::result::Result<T, Error>;
