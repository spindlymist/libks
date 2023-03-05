use crate::Result;
use std::{
    env,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    io::{BufReader, BufRead, BufWriter, Write, Read},
};
use byteorder::{LittleEndian, ReadBytesExt};

mod error;
pub use error::KnyttBinError;

const ENTRY_SIGNATURE: [u8; 2] = [b'N', b'F'];
const MAX_FILE_SIZE: usize = 1024 * 1024 * 128; // 128 MB

/// Unpacks a .knytt.bin file at `bin_path` into the directory at `output_dir`.
/// 
/// On success, it returns the number of files unpacked.
/// 
/// `output_dir` must already exist. A subdirectory will be created with the
/// specified by the .knytt.bin file.
pub fn unpack<P1, P2>(bin_path: P1, output_dir: P2) -> Result<usize>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>
{
    let mut reader = {
        let file = File::open(bin_path)?;
        BufReader::new(file)
    };

    // First header gives the name of the enclosing directory
    // It also gives a number related to the number of packed files, but which may be higher or lower
    // depending on some arcane rules in the original packer implementation, rendering it useless.
    let (dir_name, _) = read_entry_header(&mut reader)?;

    // Create the world directory and cd into it temporarily
    let prev_wd = {
        let dir_path = output_dir.as_ref().join(dir_name);
        fs::create_dir(&dir_path)?;
        
        let prev_wd = env::current_dir()?;
        env::set_current_dir(dir_path)?;

        prev_wd
    };

    // Unpack the contents
    let mut unpacked_count = 0;
    let mut buf = vec![];
    while !reader.fill_buf()?.is_empty() {
        unpack_next_entry(&mut reader, &mut buf)?;
        unpacked_count += 1;
    }

    // Restore working directory
    env::set_current_dir(prev_wd)?;

    Ok(unpacked_count)
}

/// Parses a .knytt.bin entry header from `reader`.
/// 
/// The header format is:
/// - Signature `"NF"` (2 bytes)
/// - Null-terminated file path (relative to root directory)
/// - File size (unsigned 32-bit integer)
fn read_entry_header(reader: &mut BufReader<File>) -> Result<(PathBuf, usize)> {
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
        let path: PathBuf = read_null_term_string(reader)?.into();

        if path.as_os_str().is_empty() {
            return Err(KnyttBinError::EmptyPath.into());
        }

        if path.iter().any(|component| component == "..") {
            return Err(KnyttBinError::IllegalPath(path).into());
        }

        path
    };

    // Read and validate size
    let size: usize = {
        let size = reader.read_u32::<LittleEndian>()?
            .try_into()
            .expect("u32::MAX should be less than or equal to usize::MAX");

        if size > MAX_FILE_SIZE {
            return Err(KnyttBinError::OversizedFile { path, size }.into());
        }

        size
    };

    Ok((path, size))
}

/// Unpacks the next .knytt.bin entry from `reader` into the current working directory.
fn unpack_next_entry(reader: &mut BufReader<File>, buf: &mut Vec<u8>) -> Result<()> {
    let (path, file_size) = read_entry_header(reader)?;
    
    // Prepare the buffer
    if buf.capacity() < file_size {
        *buf = Vec::with_capacity(file_size);
    }
    unsafe {
        buf.set_len(file_size);
    }

    // Read contents
    {
        let bytes_read = read_at_most(reader, buf)?;
        if bytes_read < file_size {
            return Err(KnyttBinError::MissingData {
                path, file_size, bytes_read
            }.into());
        }
    }

    // Write the contents to disk
    {
        if let Some(dir_path) = path.parent() {
            fs::create_dir_all(dir_path)?;
        }

        let mut writer = {
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            let file = options.open(path)?;
            BufWriter::new(file)
        };
        writer.write_all(buf)?;
    }

    Ok(())
}

/// Packs the files in the directory at `input_dir` into a .knytt.bin and writes it to `bin_path`.
pub fn pack<P1, P2>(_input_dir: P1, _bin_path: P2)
where
    P1: AsRef<Path>,
    P2: AsRef<Path>
{

}

/// Reads at most `buf.len()` bytes from `reader` into `buf`.
/// 
/// The return value is the number of bytes read. This is similar to `Read::read_exact` except
/// it's possible to tell how many bytes were read if EOF is reached early.
fn read_at_most(reader: &mut BufReader<File>, buf: &mut [u8]) -> Result<usize> {
    let mut reader = {
        let bytes_expected = buf.len()
            .try_into()
            .expect("usize::MAX should be less than or equal to u64::MAX");
        reader.take(bytes_expected)
    };

    let mut total_bytes_read = 0;
    let mut bytes_read = reader.read(&mut buf[total_bytes_read..])?;
    while bytes_read > 0 {
        total_bytes_read += bytes_read;
        bytes_read = reader.read(&mut buf[total_bytes_read..])?;
    };

    Ok(total_bytes_read)
}

/// Parses a Utf-8 sequence from `reader` up to the first null byte (or EOF).
/// 
/// The null byte is consumed but excluded from the returned `String`.
fn read_null_term_string(reader: &mut BufReader<File>) -> Result<String> {
    let mut buf = vec![];
    reader.read_until(0, &mut buf)?;

    let mut string = String::from_utf8(buf)?;
    string.truncate(string.len() - 1); // Discard the null byte

    Ok(string)
}
