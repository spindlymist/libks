use image::{RgbaImage, imageops};

use crate::{Result, map_bin::ScreenData, constants};

mod error;
pub use error::DrawError;

mod cache;
pub use cache::AssetCache;

pub fn tileset_index_to_pixels(i: u32) -> (u32, u32) {
    (
        (i % 16) * 24,
        (i / 16) * 24,
    )
}

pub fn screen_index_to_pixels(i: u32) -> (u32, u32) {
    (
        (i % 25) * 24,
        (i / 25) * 24,
    )
}

pub fn draw_screen(screen: &ScreenData, assets: &mut AssetCache) -> Result<RgbaImage> {
    let mut img = RgbaImage::new(600, 240);

    assets.ensure_assets_loaded(screen.assets)?;

    // draw gradient
    if let Some(gradient) = assets.get_gradient(screen.assets.gradient) {
        imageops::tile(&mut img, gradient);
    }

    // draw tile layers
    let tileset_a = assets.get_tileset(screen.assets.tileset_a);
    let tileset_b = assets.get_tileset(screen.assets.tileset_b);

    for tile_layer in &screen.layers[0..4] {
        for y in 0..constants::SCREEN_HEIGHT {
            for x in 0..constants::SCREEN_WIDTH {
                let i = x + y * constants::SCREEN_WIDTH;
                let tile = tile_layer.0[i];

                if tile.1 == 0 { continue }

                let Some(tileset) = (match tile.0 {
                    0 => tileset_a,
                    1 => tileset_b,
                    _ => None,
                }) else { continue };

                let (tile_x, tile_y) = tileset_index_to_pixels(tile.1.into());
                let tile_img = imageops::crop_imm(tileset, tile_x, tile_y, 24, 24);

                let (screen_x, screen_y) = screen_index_to_pixels(i.try_into().unwrap());

                imageops::overlay(&mut img, &*tile_img, screen_x.into(), screen_y.into());
            }
        }
    }

    Ok(img)
}
