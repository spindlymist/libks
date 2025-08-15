#[derive(thiserror::Error, Debug)]
pub enum MapBinError {
    #[error("A screen position is malformed.")]
    BadScreenPosition,
    #[error("An entry called `{entry_key}` is missing data: found {bytes_read}/{entry_len} bytes.")]
    MissingData {
        entry_key: String,
        entry_len: usize,
        bytes_read: usize,
    },
    #[error("The screen at x{}y{} is missing data.", position.0, position.1)]
    ScreenMissingData {
        position: crate::common::ScreenCoord,
    }
}
