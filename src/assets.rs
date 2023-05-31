use std::{path::PathBuf, fs::{self, File}};

use crate::Result;

type AssetId = u8;

pub struct AssetSource {
    pub data_folder: PathBuf,
    pub world_folder: PathBuf,
}

macro_rules! asset_methods {
    ( $path:ident, $open:ident, $read:ident, $fmt:tt ) => {
        pub fn $path(&self, index: AssetId) -> Option<PathBuf> {
            let rel_path = format!($fmt, index);
            self.resolve_path(rel_path)
        }

        pub fn $open(&self, index: AssetId) -> Option<Result<File>> {
            let rel_path = format!($fmt, index);
            self.open_path(rel_path)
        }

        pub fn $read(&self, index: AssetId) -> Option<Result<Vec<u8>>> {
            let rel_path = format!($fmt, index);
            self.read_path(rel_path)
        }
    };
}

impl AssetSource {
    fn resolve_path(&self, rel_path: String) -> Option<PathBuf> {
        // Try the world folder first
        {
            let world_path = self.world_folder.join(&rel_path);
            if world_path.is_file() {
                return Some(world_path);
            }
        }

        // Fall back to the data folder
        {
            let data_path = self.data_folder.join(&rel_path);
            if data_path.is_file() {
                return Some(data_path);
            }
        }

        // Asset doesn't exist
        None
    }

    fn open_path(&self, rel_path: String) -> Option<Result<File>> {
        self.resolve_path(rel_path).map(|path|
            File::open(path).map_err(|err| err.into())
        )
    }

    fn read_path(&self, rel_path: String) -> Option<Result<Vec<u8>>> {
        self.resolve_path(rel_path).map(|path|
            fs::read(path).map_err(|err| err.into())
        )
    }

    asset_methods!(ambiance_path, ambiance_open, ambiance_read, "Ambiance/Ambi{}.ogg");
    asset_methods!(music_path, music_open, music_read, "Music/Song{}.ogg");
    asset_methods!(tileset_path, tileset_open, tileset_read, "Tilesets/Tileset{}.png");
    asset_methods!(gradient_path, gradient_open, gradient_read, "Gradients/Gradient{}.png");

}
