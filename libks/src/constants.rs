#![allow(dead_code)]

pub const SCREEN_WIDTH: usize = 25;
pub const SCREEN_HEIGHT: usize = 10;
pub const TILES_PER_LAYER: usize = SCREEN_WIDTH * SCREEN_HEIGHT;
pub const LAYER_COUNT: usize = 8;

/// 1 kibibyte (2^10 bytes)
pub(crate) const KB: usize = 1024;
/// 1 mebibyte (2^20 bytes)
pub(crate) const MB: usize = 1024 * 1024;
/// 1 gibibyte (2^30 bytes)
pub(crate) const GB: usize = 1024 * 1024 * 1024;
