use std::{
    cmp::min,
    io::{self, Read, BufRead},
    path::Path,
};

use thiserror::Error;

/// Ensures `buf` has a minimum capacity of `size` bytes and sets its length to 0.
pub fn clear_buffer_and_reserve(buf: &mut Vec<u8>, size: usize) {
    // Vec::clear drops each element, which is unnecessary and slow if it doesn't get optimized away
    unsafe {
        buf.set_len(0);
    }
    buf.reserve(size);
}

/// Ensures `buf` has a minimum capacity of `size` bytes and sets its length to `size`.
/// 
/// This allows a slice to be taken up to the index `size - 1` without panicking, which
/// is useful for functions like [`std::io::Read::read`] that take `&mut [u8]`.
pub fn resize_buffer(buf: &mut Vec<u8>, size: usize) {
    clear_buffer_and_reserve(buf, size);
    unsafe {
        buf.set_len(size);
    }
}

pub fn skip_at_most<R>(reader: &mut R, buf: &mut [u8], n: usize) -> Result<usize, io::Error>
where
    R: BufRead
{
    let mut remaining = n;
    let mut skipped = 0;

    while remaining > 0 {
        let next_chunk_len = min(remaining, buf.len());
        let bytes_read = reader.read(&mut buf[..next_chunk_len])?;

        if bytes_read == 0 {
            break;
        }

        remaining -= bytes_read;
        skipped += bytes_read;
    }

    Ok(skipped)
}

/// Reads at most `buf.len()` bytes from `reader` into `buf`.
/// 
/// The return value is the number of bytes read. This is similar to [`std::io::Read::read_exact`]
/// except it's possible to tell how many bytes were read if EOF is reached early.
pub fn read_at_most<R>(reader: &mut R, buf: &mut [u8]) -> Result<usize, io::Error>
where
    R: BufRead
{
    let mut reader = {
        let bytes_expected = buf.len().try_into()
            .expect("usize::MAX should be less than or equal to u64::MAX");
        reader.take(bytes_expected)
    };

    let mut total_bytes_read = 0;
    let mut bytes_read = reader.read(buf)?;
    while bytes_read > 0 {
        total_bytes_read += bytes_read;
        bytes_read = reader.read(&mut buf[total_bytes_read..])?;
    };

    Ok(total_bytes_read)
}

#[derive(Error, Debug)]
pub enum ReadStringError {
    #[error("Failed to read string: reached max length before next null byte")]
    TooLong,
    #[error("Failed to read string: reader was at EOF")]
    Empty,
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Decodes Windows-1252-encoded bytes from `reader` up to the first null byte (or EOF). The
/// null byte is consumed but not returned.
pub fn read_windows_1252_null_term<R: BufRead>(
    reader: &mut R,
    buf: &mut Vec<u8>,
    max_len: usize
) -> Result<String, ReadStringError> {
    use encoding_rs::WINDOWS_1252;

    let mut reader = {
        let max_len: u64 = max_len.try_into()
            .expect("usize::MAX should be less than or equal to u64::MAX");
        reader.take(max_len)
    };

    clear_buffer_and_reserve(buf, max_len);
    reader.read_until(0, buf)?;

    // Check the last byte
    match buf.pop() {
        Some(0) => (),
        Some(_) => return Err(ReadStringError::Empty),
        None => return Err(ReadStringError::TooLong),
    }

    // encoding_rs::WINDOWS_1252 can't have any decoding errors since it is a single byte encoding and every character
    // maps to a Unicode codepoint. See https://encoding.spec.whatwg.org/windows-1252.html
    let (string, _) = WINDOWS_1252.decode_without_bom_handling(buf);
    
    Ok(string.to_string())
}

pub enum PathInfo {
    NonemptyDirectory,
    EmptyDirectory,
    File,
    Symlink,
    Other,
    Nonexistent,
}

/// Determines what kind of file system entry exists at `path`, if any.
/// Discriminates between empty and nonempty directories.
pub fn path_info<P>(path: P) -> Result<PathInfo, io::Error>
where
    P: AsRef<Path>
{
    let path = path.as_ref();
    match path.symlink_metadata() {
        Ok(meta) if meta.is_dir() => {
            match path.read_dir()?.next() {
                Some(_) => Ok(PathInfo::NonemptyDirectory),
                None => Ok(PathInfo::EmptyDirectory),
            }
        },
        Ok(meta) if meta.is_file() => {
            Ok(PathInfo::File)
        },
        Ok(meta) if meta.is_symlink() => {
            Ok(PathInfo::Symlink)
        },
        Ok(_) => Ok(PathInfo::Other),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            Ok(PathInfo::Nonexistent)
        },
        Err(err) => Err(err),
    }
}
