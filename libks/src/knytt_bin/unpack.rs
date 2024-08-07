use std::{
    env,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    io::{BufReader, BufRead, BufWriter, Write, Read},
};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::Result;
use super::{KnyttBinError, ENTRY_SIGNATURE, MB};

/// Configures the behavior of [`unpack_with_options`].
#[derive(Debug)]
pub struct UnpackOptions {
    /// If `true`, the output directory is deleted prior to unpacking if it exists
    /// and is not empty. Otherwise, an error is returned. Defaults to `false`.
    pub allow_overwrite: bool,
    /// If `true`, the enclosing directory specified in the .knytt.bin will be created
    /// inside the output directory. Otherwise, the files will be unpacked directly
    /// into the output directory. Defaults to `true`.
    pub create_top_level_dir: bool,
    /// The maximum size in bytes allowed for a single unpacked file. Defaults to 256 MiB.
    pub max_file_size: usize,
    /// The maximum length in bytes allow for a single file path. Defaults to 256.
    pub max_path_len: usize,
}

impl Default for UnpackOptions {
    fn default() -> Self {
        Self {
            allow_overwrite: false,
            create_top_level_dir: true,
            max_file_size: 256 * MB,
            max_path_len: 256,
        }
    }
}

/// Unpacks a .knytt.bin file at `bin_path` into a subdirectory of `output_dir`.
/// The name of the subdirectory is specified in the .knytt.bin data.
/// 
/// On success, it returns the directory that the files were unpacked into.
/// 
/// The default unpacking options will be used. See [`UnpackOptions`] for more information.
/// If you need to override them, use [`unpack_with_options`].
pub fn unpack<P1, P2>(bin_path: P1, output_dir: P2) -> Result<PathBuf>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    unpack_with_options(bin_path, output_dir, UnpackOptions::default())
}

/// Unpacks a .knytt.bin file at `bin_path` into the directory at `output_dir`
/// or a subdirectory thereof.
/// 
/// On success, it returns the directory that the files were unpacked into.
pub fn unpack_with_options<P1, P2>(bin_path: P1, output_dir: P2, options: UnpackOptions) -> Result<PathBuf>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>
{
    let mut reader = {
        let file = File::open(bin_path)?;
        BufReader::new(file)
    };
    let mut buf = Vec::<u8>::with_capacity(4 * MB);

    // First header gives the name of the enclosing directory
    // It also gives a number related to the number of packed files, but which may be higher or lower
    // depending on some arcane rules in the original packer implementation, rendering it useless.
    let (level_name, _) = read_entry_header(&mut reader, &mut buf, options.max_path_len)?;

    // Determine the final output directory
    let output_dir =
        if options.create_top_level_dir {
            output_dir.as_ref().join(level_name)
        }
        else {
            output_dir.as_ref().to_owned()
        };

    // Check if the output path exists and create if necessary
    match path_info(&output_dir)? {
        PathInfo::NonemptyDirectory if options.allow_overwrite => {
            fs::remove_dir_all(&output_dir)?;
            fs::create_dir_all(&output_dir)?;
        },
        PathInfo::NonemptyDirectory => {
            return Err(KnyttBinError::UnauthorizedOverwrite(output_dir).into());
        },
        PathInfo::EmptyDirectory => (),
        PathInfo::Nonexistent => {
            fs::create_dir_all(&output_dir)?;
        },
        _ => {
            return Err(KnyttBinError::OutputPathExists(output_dir).into());
        },
    };

    // cd into the world directory temporarily
    let prev_working_dir = env::current_dir()?;
    env::set_current_dir(&output_dir)?;

    // Unpack the contents
    while !reader.fill_buf()?.is_empty() {
        unpack_next_entry(&mut reader, &mut buf, options.max_path_len, options.max_file_size)?;
    }

    // Restore working directory
    env::set_current_dir(prev_working_dir)?;

    Ok(output_dir)
}

/// Parses a .knytt.bin entry header from `reader`.
/// 
/// The header format is:
/// - Signature `"NF"` (2 bytes)
/// - Null-terminated file path (relative to root directory)
/// - File size (unsigned 32-bit integer)
fn read_entry_header(
    reader: &mut BufReader<File>, 
    buf: &mut Vec<u8>,
    max_path_len: usize,
) -> Result<(PathBuf, usize)> {
    // Validate entry signature
    {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf)?;
        if buf != ENTRY_SIGNATURE {
            return Err(KnyttBinError::UnrecognizedSignature(buf).into());
        }
    }

    // Read and validate path
    let path: PathBuf = {
        let path = read_null_term_string(reader, buf, max_path_len)?;

        if path.is_empty() {
            return Err(KnyttBinError::EmptyPath.into());
        }

        let path = PathBuf::from(path);

        if path.is_absolute()
            || path.iter().any(|part| part == "..")
        {
            return Err(KnyttBinError::IllegalPath(path).into());
        }

        path
    };

    // Read and validate size
    let size: usize = reader.read_u32::<LittleEndian>()?
        .try_into()
        .expect("u32::MAX should be less than or equal to usize::MAX");

    Ok((path, size))
}

/// Unpacks the next .knytt.bin entry from `reader` into the current working directory.
fn unpack_next_entry(
    reader: &mut BufReader<File>,
    buf: &mut Vec<u8>,
    max_path_len: usize,
    max_file_size: usize,
) -> Result<()> {
    let (path, file_size) = read_entry_header(reader, buf, max_path_len)?;

    if file_size > max_file_size {
        return Err(KnyttBinError::OversizedFile {
            path,
            size: file_size,
        }.into());
    }

    // Read contents
    {
        resize_buffer(buf, file_size);
        let bytes_read = read_at_most(reader, buf.as_mut_slice())?;
        if bytes_read < file_size {
            return Err(KnyttBinError::MissingData {
                path,
                file_size,
                bytes_read,
            }.into());
        }
    }

    // Write the contents to disk
    {
        if let Some(parent) = path.parent() {
            if parent.iter().next().is_some() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut writer = {
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)?;
            BufWriter::new(file)
        };
        writer.write_all(buf)?;
    }

    Ok(())
}

/// Ensures `buf` has a minimum capacity of `size` bytes and sets its length to 0.
fn clear_buffer_and_reserve(buf: &mut Vec<u8>, size: usize) {
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
fn resize_buffer(buf: &mut Vec<u8>, size: usize) {
    clear_buffer_and_reserve(buf, size);
    unsafe {
        buf.set_len(size);
    }
}

/// Reads at most `buf.len()` bytes from `reader` into `buf`.
/// 
/// The return value is the number of bytes read. This is similar to [`std::io::Read::read_exact`]
/// except it's possible to tell how many bytes were read if EOF is reached early.
fn read_at_most(reader: &mut BufReader<File>, buf: &mut [u8]) -> Result<usize> {
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

/// Parses a Windows-1252 sequence from `reader` up to the first null byte (or EOF).
/// 
/// The null byte is consumed but excluded from the returned `String`.
fn read_null_term_string(reader: &mut BufReader<File>, buf: &mut Vec<u8>, max_len: usize) -> Result<String> {
    use encoding_rs::WINDOWS_1252;

    let mut reader = {
        let max_len: u64 = max_len.try_into()
            .expect("usize::MAX should be less than or equal to u64::MAX");
        reader.take(max_len)
    };

    clear_buffer_and_reserve(buf, max_len);
    reader.read_until(0, buf)?;

    let (string, had_errors) = WINDOWS_1252.decode_without_bom_handling(buf);
    let mut string = string.to_string();

    if had_errors {
        return Err(KnyttBinError::BadEncoding(string).into());
    }

    string.truncate(string.len() - 1); // Discard the null byte

    Ok(string)
}

enum PathInfo {
    NonemptyDirectory,
    EmptyDirectory,
    File,
    Symlink,
    Other,
    Nonexistent,
}

/// Determines what kind of file system entry exists at `path`, if any.
/// Discriminates between empty and nonempty directories.
fn path_info<P>(path: P) -> Result<PathInfo>
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
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            Ok(PathInfo::Nonexistent)
        },
        Err(err) => Err(err.into()),
    }
}
