use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum WorldIniError {
    #[error("The World.ini at `{path:?}` was not encoded properly (expected Windows-1252).")]
    BadEncoding {
        path: PathBuf,
    },
}
