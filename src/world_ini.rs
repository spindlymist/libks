use std::path::Path;

use ini::Ini;

use crate::Result;

mod error;
pub use error::WorldIniError;

/// Attempts to read and parse the World.ini for the level in `world_dir`.
pub fn load_ini(world_dir: &Path) -> Result<Ini> {
    let ini_path = world_dir.join("World.ini");
    match Ini::load_from_file(&ini_path) {
        Ok(ini) => Ok(ini),
        Err(err) => match err {
            ini::Error::Io(source) => Err(source.into()),
            ini::Error::Parse(source) => Err(WorldIniError::BadWorldIni {
                source,
                path: ini_path,
            }.into()),
        },
    }
}
