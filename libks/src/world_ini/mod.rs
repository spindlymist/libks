use std::{fs, path::Path};

use libks_ini::Ini;

use crate::Result;

mod error;
pub use error::WorldIniError;

/// Attempts to read and parse the World.ini for the level in `world_dir`.
pub fn load_ini(world_dir: &Path) -> Result<Ini> {
    let ini_path = world_dir.join("World.ini");
    let ini_contents = {
        let bytes = fs::read(&ini_path)?;
        let (contents, _, had_errors) = encoding_rs::WINDOWS_1252.decode(&bytes);

        if had_errors {
            return Err(WorldIniError::BadEncoding {
                path: ini_path,
            }.into());
        }

        contents.to_string()
    };

    Ok(Ini::new(&ini_contents))
}
