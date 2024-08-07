mod error;
pub use error::KnyttBinError;

mod pack;
pub use pack::pack;

mod unpack;
pub use unpack::{
    unpack,
    unpack_with_options,
    UnpackOptions,
};

const ENTRY_SIGNATURE: [u8; 2] = [b'N', b'F'];
const MB: usize = 1024 * 1024;
