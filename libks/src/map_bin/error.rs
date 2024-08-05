#[derive(thiserror::Error, Debug)]
pub enum MapBinError {
    #[error("A screen position is malformed.")]
    BadScreenPosition,
    #[error("The screen at x{}y{} has an unrecognized signature.", position.0, position.1)]
    UnrecognizedSignature {
        position: (i64, i64),
        bytes: [u8; 4],
    },
    #[error("The screen at x{}y{} is missing data.", position.0, position.1)]
    MissingData {
        position: (i64, i64),
    }
}
