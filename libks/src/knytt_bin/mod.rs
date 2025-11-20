mod error;
pub use error::KnyttBinError;

mod pack;
pub use pack::pack;

mod unpack;
pub use unpack::{
    unpack,
    unpack_with_options,
    parse_headers,
    UnpackOptions,
};

const ENTRY_SIGNATURE: [u8; 2] = [b'N', b'F'];
