#![feature(error_generic_member_access)]
#![feature(provide_any)]

pub mod constants;

pub mod knytt_bin;
pub use knytt_bin::KnyttBinError;

pub mod map_bin;
pub use map_bin::MapBinError;

pub mod error;
pub use error::Error;
pub use error::Result;
