use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum WorldIniError {
    #[error("The World.ini file at `{path:?}` could not be parsed.")]
    BadWorldIni {
        source: ini::ParseError,
        path: PathBuf,
    },
    #[error("The World.ini at `{path:?}` was not encoded properly (expected Windows-1252).")]
    BadEncoding {
        path: PathBuf,
    },
}
