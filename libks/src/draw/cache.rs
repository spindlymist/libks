use std::collections::{HashMap, hash_map::Entry};

use image::{io::Reader as ImageReader, DynamicImage};

use crate::{Result, map_bin::{AssetId, Tile, AssetIds}, assets::AssetSource};
use super::DrawError;

pub struct AssetCache {
    source: AssetSource,
    tilesets: HashMap<AssetId, Option<DynamicImage>>,
    gradients: HashMap<AssetId, Option<DynamicImage>>,
    objects: HashMap<Tile, Option<DynamicImage>>,
}

impl AssetCache {
    pub fn new(source: AssetSource) -> AssetCache {
        AssetCache {
            source,
            tilesets: HashMap::new(),
            gradients: HashMap::new(),
            objects: HashMap::new(),
        }
    }

    pub fn get_tileset(&self, id: AssetId) -> Option<&DynamicImage> {
        self.tilesets.get(&id)
            .unwrap_or(&None)
            .as_ref()
    }

    pub fn get_gradient(&self, id: AssetId) -> Option<&DynamicImage> {
        self.gradients.get(&id)
            .unwrap_or(&None)
            .as_ref()
    }

    pub fn get_object(&mut self, tile: Tile) -> Option<&DynamicImage> {
        self.objects.get(&tile)
            .unwrap_or(&None)
            .as_ref()
    }

    pub fn ensure_assets_loaded(&mut self, assets: AssetIds) -> Result<()> {
        self.ensure_tileset_loaded(assets.tileset_a)?;
        self.ensure_tileset_loaded(assets.tileset_b)?;
        self.ensure_gradient_loaded(assets.gradient)?;

        Ok(())
    }

    pub fn ensure_tileset_loaded(&mut self, id: AssetId) -> Result<()> {
        if let Entry::Vacant(entry) = self.tilesets.entry(id) {
            let Some(path) = self.source.tileset_path(id) else {
                entry.insert(None);
                return Ok(());
            };

            match ImageReader::open(&path)?.decode() {
                Ok(img) => entry.insert(Some(img)),
                Err(source) => return Err(DrawError::Image {
                    source,
                    path,
                }.into()),
            };
        }

        Ok(())
    }

    pub fn ensure_gradient_loaded(&mut self, id: AssetId) -> Result<()> {
        if let Entry::Vacant(entry) = self.gradients.entry(id) {
            let Some(path) = self.source.gradient_path(id) else {
                entry.insert(None);
                return Ok(());
            };

            match ImageReader::open(&path)?.decode() {
                Ok(img) => entry.insert(Some(img)),
                Err(source) => return Err(DrawError::Image {
                    source,
                    path,
                }.into()),
            };
        }

        Ok(())
    }
}
