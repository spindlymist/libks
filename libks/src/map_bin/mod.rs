use std::{
    cmp::min,
    fs::OpenOptions,
    io::{self, prelude::*, BufReader, BufWriter},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};

use crate::{
    common::{parse_xy, ScreenCoord},
    constants::*,
    error::KsError,
    io_util,
    Result,
};

mod error;
pub use error::MapBinError;

const SCREEN_DATA_LEN: usize = 3006;
const SCREEN_DATA_LEN_U32: u32 = 3006;

#[derive(Debug, Clone)]
pub struct ScreenData {
    pub position: ScreenCoord,
    pub layers: [LayerData; LAYER_COUNT],
    pub assets: AssetIds,
}

pub type AssetId = u8;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetIds {
    pub tileset_a: AssetId,
    pub tileset_b: AssetId,
    pub ambiance_a: AssetId,
    pub ambiance_b: AssetId,
    pub music: AssetId,
    pub gradient: AssetId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tile(pub u8, pub u8);

#[derive(Debug, Clone)]
pub struct LayerData(pub [Tile; TILES_PER_LAYER]);

pub enum ParseWarning {
    UnrecognizedEntry(String, usize),
    IncompleteScreenData(String, usize),
    ExtraScreenData(String, usize),
}

impl std::fmt::Display for ParseWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParseWarning::*;
        match self {
            UnrecognizedEntry(key, len) =>
                write!(f, "Found an unrecognized entry `{key}` with {len} bytes."),
            IncompleteScreenData(key, len) =>
                write!(f, "The screen entry `{key}` was skipped because it was only {len}/3006 bytes."),
            ExtraScreenData(key, len) =>
                write!(f, "The screen entry `{key}` had {} extra bytes.", len - 3006),
        }
    }
}

/// Parses all screens from the Map.bin data stored at `path`. The data is assumed to be gzipped.
/// 
/// This variant ignores abnormalities in the data. Use [`parse_map_file_with_warnings`] if you
/// want information about abnormalities.
pub fn parse_map_file<P>(path: P) -> Result<Vec<ScreenData>>
where
    P: AsRef<Path>
{
    Ok(parse_map_file_with_warnings(path)?.0)
}

/// Parses all screens from the Map.bin data stored at `path`. The data is assumed to be gzipped.
/// 
/// This variant provides warnings if there are abnormalities in the data such as non-screen entries
/// or screens with extra data. If you don't care about these warnings, use [`parse_map_file`].
pub fn parse_map_file_with_warnings<P>(path: P) -> Result<(Vec<ScreenData>, Vec<ParseWarning>)>
where
    P: AsRef<Path>
{
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    parse_map_gzipped(reader)
}

/// Parses all screens from `reader`, which must yield gzipped Map.bin data.
/// If the data is uncompressed, call [`parse_map_uncompressed`] instead.
pub fn parse_map_gzipped<R>(reader: R) -> Result<(Vec<ScreenData>, Vec<ParseWarning>)>
where
    R: Read
{
    let decoder = GzDecoder::new(reader);
    let reader = BufReader::new(decoder);
    parse_map_uncompressed(reader)
}

/// Parses all screens from `reader`, which must yield uncompressed Map.bin data.
/// If the data is compressed, call [`parse_map_gzipped`] instead.
/// 
/// Map.bin consists solely of a series of named binary chunks called workspaces. Each
/// workspace consists of:
/// - A name, such as `x1000y1000`. Null-terminated string. The encoding is presumed
///   to be Windows-1252, but this hasn't been confirmed.
/// - Length in bytes. Little endian 32-byte integer. Presumed to be unsigned, but
///   this hasn't been confirmed.
/// - Data
pub fn parse_map_uncompressed<R>(mut reader: R) -> Result<(Vec<ScreenData>, Vec<ParseWarning>)>
where
    R: BufRead
{
    let mut warnings = Vec::new();
    let mut screens = Vec::new();
    let mut buf = Vec::with_capacity(256);

    let mut warn = |warning| warnings.push(warning);
    
    // Parse screens
    while !reader.fill_buf()?.is_empty() {
        let (entry_key, entry_len) = read_entry_header(&mut reader, &mut buf, 256)?;

        let bytes_read = match parse_xy(&entry_key) {
            // Incomplete screen data
            Some(_) if entry_len < SCREEN_DATA_LEN => {
                warn(ParseWarning::IncompleteScreenData(entry_key.clone(), entry_len));
                0
            },
            // Screen data
            Some(position) => {
                if entry_len > SCREEN_DATA_LEN {
                    warn(ParseWarning::ExtraScreenData(entry_key.clone(), entry_len));
                }

                let screen = parse_screen(&mut reader, position)?;
                screens.push(screen);

                SCREEN_DATA_LEN
            },
            // Unknown entry
            // This is most likely level editor garbage under the empty key
            None => {
                warn(ParseWarning::UnrecognizedEntry(entry_key.clone(), entry_len));
                0
            }
        };

        let bytes_to_skip = entry_len - bytes_read;
        if bytes_to_skip > 0 {
            // Generally, this won't happen, but when it does, we may need to
            // skip a lot of bytes. We'll enlarge the buffer as needed (up to 1 MB)
            // to speed things up.
            io_util::resize_buffer(&mut buf, min(bytes_to_skip, MB));

            let bytes_skipped = io_util::skip_at_most(&mut reader, &mut buf, bytes_to_skip)?;
            if bytes_skipped < bytes_to_skip {
                return Err(MapBinError::MissingData {
                    entry_key,
                    entry_len,
                    bytes_read: bytes_read + bytes_skipped,
                }.into());
            }
        }
    }

    Ok((screens, warnings))
}

fn read_entry_header<R>(reader: &mut R, buf: &mut Vec<u8>, max_len: usize) -> Result<(String, usize)>
where
    R: BufRead
{
    let key = io_util::read_windows_1252_null_term(reader, buf, max_len)?;
    let len = reader.read_u32::<LittleEndian>()?
        .try_into()
        .expect("u32::MAX should be less than or equal to usize::MAX");

    Ok((key, len))
}

/// Parses a single screen from `reader`.
/// 
/// The screen format is:
/// - 4 tile layers (0-3, 250 bytes each) - see [`parse_tile_layer`]
/// - 4 object layers (4-7, 500 bytes each) - see [`parse_object_layer`]
/// - Asset IDs (6 bytes) - see [`parse_asset_ids`]
fn parse_screen<R>(reader: &mut R, position: ScreenCoord) -> Result<ScreenData>
where
    R: BufRead
{
    // Read layers
    let mut layers: [LayerData; LAYER_COUNT] = unsafe { std::mem::zeroed() };
    for (i, layer) in layers.iter_mut().enumerate() {
        if is_object_layer(i) {
            *layer = parse_object_layer(reader)
                .map_err(|err| make_missing_data_error(err, position))?;
        }
        else {
            *layer = parse_tile_layer(reader)
                .map_err(|err| make_missing_data_error(err, position))?;
        }
    }

    // Read assets
    let assets = parse_asset_ids(reader)
        .map_err(|err| make_missing_data_error(err, position))?;

    Ok(ScreenData {
        position,
        layers,
        assets,
    })
}

/// Converts an `UnexpectedEof` error to `MapBinError::MissingData`.
fn make_missing_data_error(err: KsError, position: ScreenCoord) -> KsError {
    if let KsError::Io { source, .. } = &err {
        if source.kind() == io::ErrorKind::UnexpectedEof {
            return MapBinError::ScreenMissingData { position }.into();
        }
    }

    err
}

/// Returns true if the layer at index `i` is an object layer.
fn is_object_layer(i: usize) -> bool {
    i >= 4
}

/// Parses an asset ID block from `reader`.
/// 
/// Each asset ID is a single unsigned byte. The order is:
/// tileset A, tileset B, music, ambiance A, ambiance B, gradient.
fn parse_asset_ids<R>(reader: &mut R) -> Result<AssetIds>
where
    R: Read
{
    Ok(AssetIds {
        tileset_a: reader.read_u8()?,
        tileset_b: reader.read_u8()?,
        ambiance_a: reader.read_u8()?,
        ambiance_b: reader.read_u8()?,
        music: reader.read_u8()?,
        gradient: reader.read_u8()?,
    })
}

/// Parses a single tile layer from `reader`.
/// 
/// A tile layer consists of 250 bytes. Each byte represents one tile, starting from the top left
/// and proceeding row by row. The highest order bit is 0 for tileset A or 1 for tileset B.
/// The remaining 7 bits are the tile index, again starting from the top left and proceeding row by row.
/// 
/// For example, `0x00` is the top left tile of tileset A. `0x01` is the tile to its right.
/// `0x7F` is the bottom right tile of tileset A. `0x80` is the top left tile of tileset B.
/// `0x81` is the tile to its right. `0xFF` is the bottom right tile of tileset B.
fn parse_tile_layer<R>(reader: &mut R) -> Result<LayerData>
where
    R: Read
{
    let mut raw = [0; TILES_PER_LAYER];
    reader.read_exact(&mut raw)?;

    let mut tiles: [Tile; TILES_PER_LAYER] = unsafe { std::mem::zeroed() };
    for (i, tile) in tiles.iter_mut().enumerate() {
        if raw[i] < 128 {
            *tile = Tile(0, raw[i]);
        }
        else {
            *tile = Tile(1, raw[i] - 128);
        }
    }

    Ok(LayerData(tiles))
}

/// Parses a single object layer from `reader`.
/// 
/// An object layer consists of 500 bytes: 250 bytes of object indices followed by 250 bytes
/// of bank indices. In each 250 byte block, each byte represents one tile, starting from the
/// top left and proceeding row by row.
fn parse_object_layer<R>(reader: &mut R) -> Result<LayerData>
where
    R: Read
{
    let mut indices = [0; TILES_PER_LAYER];
    reader.read_exact(&mut indices)?;

    let mut banks = [0; TILES_PER_LAYER];
    reader.read_exact(&mut banks)?;

    let mut tiles: [Tile; TILES_PER_LAYER] = unsafe { std::mem::zeroed() };
    for (i, tile) in tiles.iter_mut().enumerate() {
        *tile = Tile(banks[i], indices[i]);
    }

    Ok(LayerData(tiles))
}

/// Compresses and writes the data in `screens` to the file at `path`.
pub fn write_map_file<P>(path: P, screens: &Vec<ScreenData>) -> Result<()>
where
    P: AsRef<Path>
{
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = GzEncoder::new(writer, Compression::default());

    let mut screen_buffer: [u8; 3006] = [0; 3006];
    for screen in screens {
        let mut i = 0;

        for layer_index in 0..4 {
            for tile in &screen.layers[layer_index].0 {
                screen_buffer[i] = tile.1 | (tile.0 * 0x80);
                i += 1;
            }
        }
        
        for layer_index in 4..8 {
            for tile in &screen.layers[layer_index].0 {
                screen_buffer[i] = tile.1;
                screen_buffer[i + 250] = tile.0;
                i += 1;
            }
            i += 250;
        }

        screen_buffer[i]     = screen.assets.tileset_a;
        screen_buffer[i + 1] = screen.assets.tileset_b;
        screen_buffer[i + 2] = screen.assets.ambiance_a;
        screen_buffer[i + 3] = screen.assets.ambiance_b;
        screen_buffer[i + 4] = screen.assets.music;
        screen_buffer[i + 5] = screen.assets.gradient;
        
        encoder.write_all(&format!("x{}y{}\0", screen.position.0, screen.position.1).into_bytes())?;
        encoder.write_u32::<LittleEndian>(SCREEN_DATA_LEN_U32)?;
        encoder.write_all(&screen_buffer)?;
        encoder.flush()?;
    }

    Ok(())
}
