use std::{io::{self, prelude::*, BufReader, BufWriter}, path::Path, fs::OpenOptions};
use byteorder::ReadBytesExt;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};

use crate::{Result, constants::*, error::KsError};

mod error;
pub use error::MapBinError;

#[derive(Debug, Clone)]
pub struct ScreenData {
    pub position: (i64, i64),
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

/// Parses all screens from the Map.bin data stored at `path`. The data is assumed to be gzipped.
pub fn parse_map_file<P>(path: P) -> Result<Vec<ScreenData>>
where
    P: AsRef<Path>
{
    let file = std::fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    parse_map_gzipped(&mut reader)
}

/// Parses all screens from `reader`, which must yield gzipped Map.bin data.
/// If the data is uncompressed, call [`parse_map_uncompressed`] instead.
pub fn parse_map_gzipped<R>(reader: &mut R) -> Result<Vec<ScreenData>>
where
    R: Read
{
    let decoder = GzDecoder::new(reader);
    let mut reader = BufReader::new(decoder);
    parse_map_uncompressed(&mut reader)
}

/// Parses all screens from `reader`, which must yield uncompressed Map.bin data.
/// If the data is compressed, call [`parse_map_gzipped`] instead.
pub fn parse_map_uncompressed<R>(reader: &mut R) -> Result<Vec<ScreenData>>
where
    R: BufRead
{
    // The level editor sometimes writes garbage before the first screen,
    // so discard anything before the first 'x'
    consume_until(reader, b'x')?;

    // Parse screens
    let mut screens = Vec::new();
    let mut buf = Vec::new();
    while !reader.fill_buf()?.is_empty() {
        let screen = parse_screen(reader, &mut buf)?;
        screens.push(screen);
        buf.clear();
    }

    Ok(screens)
}

const SCREEN_SIGNATURE: [u8; 4] = [0xBE, 0x0B, 0x00, 0x00];

/// Parses a single screen from `reader`.
/// 
/// The screen format is:
/// - Screen position as a null-terminated ASCII string, such as x1000y1000
/// - 4-byte signature (# of bytes of data to follow, which is constant)
/// - 4 tile layers (0-3, 250 bytes each) - see [`parse_tile_layer`]
/// - 4 object layers (4-7, 500 bytes each) - see [`parse_object_layer`]
/// - Asset IDs (6 bytes) - see [`parse_asset_ids`]
pub fn parse_screen<R>(reader: &mut R, buf: &mut Vec<u8>) -> Result<ScreenData>
where
    R: BufRead
{
    // Read and parse position
    let position = {
        if reader.read_until(0, buf)? < 5 {
            return Err(MapBinError::BadScreenPosition.into());
        }
        buf.pop(); // Discard the null byte

        let position = std::str::from_utf8(buf)?
            .strip_prefix('x')
            .and_then(|rest| rest.split_once('y'));
        
        if let Some((x_string, y_string)) = position {
            (str::parse::<i64>(x_string)?, str::parse::<i64>(y_string)?)
        }
        else {
            return Err(MapBinError::BadScreenPosition.into());
        }
    };

    // Verify signature
    {
        let mut bytes = [0; 4];
        reader.read_exact(&mut bytes)?;

        if bytes != SCREEN_SIGNATURE {
            return Err(MapBinError::UnrecognizedSignature { position, bytes }.into())
        }
    }

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
fn make_missing_data_error(err: KsError, position: (i64, i64)) -> KsError {
    if let KsError::Io { source, .. } = &err {
        if source.kind() == io::ErrorKind::UnexpectedEof {
            return MapBinError::MissingData { position }.into();
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
pub fn parse_asset_ids<R>(reader: &mut R) -> Result<AssetIds>
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
pub fn parse_tile_layer<R>(reader: &mut R) -> Result<LayerData>
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
pub fn parse_object_layer<R>(reader: &mut R) -> Result<LayerData>
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
        encoder.write_all(&SCREEN_SIGNATURE)?;
        encoder.write_all(&screen_buffer)?;
        encoder.flush()?;
    }

    Ok(())
}

/// Consumes bytes from `reader` up to and including a delimiter byte.
/// 
/// On success, it returns the number of bytes consumed (including the delimiter).
/// 
/// # Errors
/// 
/// If this function encounters an "end of file" before finding the delimiter, it returns
/// an [`Error::Io`] error with source error of kind `std::io::ErrorKind::UnexpectedEof`.
/// It may also return other kinds of I/O errors.
fn consume_until<R>(reader: &mut R, delim: u8) -> Result<usize>
where
    R: BufRead
{
    let mut consumed_count = 0;
    let mut bytes: &[u8] = &[];
    let mut index = None;

    while index.is_none() {
        let bytes_len = bytes.len();
        reader.consume(bytes_len);
        consumed_count += bytes_len;

        bytes = reader.fill_buf()?;
        if bytes.is_empty() {
            return Err(
                io::Error::new(io::ErrorKind::UnexpectedEof, "consume_until reached EOF before finding delimiter.").into()
            );
        }

        index = bytes.iter().position(|&byte| byte == delim);
    }

    let index = index.expect("Loop shouldn't end unless index is Some");

    reader.consume(index);
    consumed_count += index;

    Ok(consumed_count)
}
