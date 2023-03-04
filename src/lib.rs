#![feature(error_generic_member_access)]
#![feature(provide_any)]

pub mod knytt_bin;
pub use knytt_bin::KnyttBinError;

pub mod error;
pub use error::Error;
pub use error::Result;
