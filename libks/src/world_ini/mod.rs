use std::{borrow::Cow, fs, path::Path};

use libks_ini::edit::Ini;

use crate::Result;

mod error;
pub use error::WorldIniError;

/// Attempts to read and parse the ini at `path`.
pub fn load_ini<P>(path: P) -> Result<Ini>
where
    P: AsRef<Path>
{
    let bytes = fs::read(path)?;
    Ok(load_ini_from_bytes(bytes))
}

/// Attempts to read and parse the World.ini for the level in `world_dir`.
pub fn load_ini_from_dir<P>(world_dir: P) -> Result<Ini>
where
    P: AsRef<Path>
{
    load_ini(world_dir.as_ref().join("World.ini"))
}

/// Decodes and parses a Windows-1252 encoded ini.
pub fn load_ini_from_bytes(bytes: Vec<u8>) -> Ini {
    let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
    let source = match decoded {
        Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(bytes) },
        Cow::Owned(s) => s,
    };
    Ini::parse(source)
}

/// Converts the ini to a Windows-1252 encoded string.
pub fn encode_ini(ini: &Ini) -> Vec<u8> {
    let contents_utf8 = ini.to_string();
    let (contents_win1252, _, _) = encoding_rs::WINDOWS_1252.encode(&contents_utf8);
    match contents_win1252 {
        Cow::Borrowed(_) => contents_utf8.into_bytes(),
        Cow::Owned(bytes) => bytes
    }
}

/// Encodes and writes the ini to `path`.
pub fn write_ini<P: AsRef<Path>>(ini: &Ini, path: P) -> Result<()> {
    let contents = encode_ini(ini);
    fs::write(path, contents)?;
    Ok(())
}
