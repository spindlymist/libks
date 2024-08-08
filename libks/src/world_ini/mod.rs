use std::{fs, path::Path};

use libks_ini::Ini;

use crate::Result;

mod error;
pub use error::WorldIniError;

/// Attempts to read and parse the World.ini for the level in `world_dir`.
pub fn load_ini<P>(ini_path: P) -> Result<Ini>
where
    P: AsRef<Path>
{
    let ini_path = ini_path.as_ref();
    let ini_contents = {
        let bytes = fs::read(ini_path)?;
        let (contents, _, had_errors) = encoding_rs::WINDOWS_1252.decode(&bytes);

        if had_errors {
            return Err(WorldIniError::BadEncoding {
                path: ini_path.to_owned(),
            }.into());
        }

        contents.to_string()
    };

    Ok(Ini::new(&ini_contents))
}

/// Attempts to read and parse the World.ini for the level in `world_dir`.
pub fn load_ini_from_dir<P>(world_dir: P) -> Result<Ini>
where
    P: AsRef<Path>
{
    load_ini(world_dir.as_ref().join("World.ini"))
}
