use crate::Result;
use std::{
    env,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    io::{BufReader, BufRead, BufWriter, Write, Read, SeekFrom, Seek},
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

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
/// 
/// The .knytt.bin's "enclosing directory" will be the name of `input_dir`.
pub fn pack<P1, P2>(input_dir: P1, bin_path: P2) -> Result<usize>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>
{
    let mut writer = {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(bin_path)?;
        BufWriter::new(file)
    };

    // Temporarily cd into the directory to be packed
    let prev_wd = env::current_dir()?;
    env::set_current_dir(input_dir)?;
    
    // First header gives the name of the enclosing directory and the number of files packed
    // We don't know how many files are going to be packed, so write a placeholder for now
    let enclosing_dir = name_of_current_dir()?;
    write_entry_header(&mut writer, &enclosing_dir, 0)?;

    // Pack it up!
    let packed_count = pack_dir_recursive("".to_owned(), &mut writer)?;

    // Go back and update the number of packed files
    writer.seek(SeekFrom::Start(0))?;
    write_entry_header(&mut writer, &enclosing_dir, packed_count)?;

    // Restore working directory
    env::set_current_dir(prev_wd)?;
    
    Ok(packed_count)
}

fn pack_dir_recursive(path: String, writer: &mut BufWriter<File>) -> Result<usize> {
    let path_ref: &Path = path.as_ref();
    let mut packed_count = 0;

    for entry in path_ref.read_dir()? {
        let entry = entry?;
        let entry_path = {
            let name = entry.file_name()
                .into_string()
                .map_err(|_| KnyttBinError::BadFileName(entry.path()))?;

            if path.is_empty() {
                name
            }
            else {
                format!("{path}/{name}")
            }
        };
        let entry_path_ref: &Path = entry_path.as_ref();

        if entry_path_ref.is_dir() {
            packed_count += pack_dir_recursive(entry_path, writer)?;
        }
        else {
            pack_file(entry_path, writer)?;
            packed_count += 1;
        }
    }

    Ok(packed_count)
}

fn pack_file(path: String, writer: &mut BufWriter<File>) -> Result<()>
{
    // Read file and determine size
    // I would like to use fs::metadata() to determine size and then io::copy to copy
    // the contents directly into the output file, but I don't want to deal with
    // platform differences. Alternatively, it would be possible to use io::copy,
    // seek back to the file size offset, write the size returned by io::copy, and then
    // seek to the end, but that is probably not worth it. Most files being packed
    // are not going to be very large.
    let contents = fs::read(&path)?;
    let file_size = contents.len();

    // Write header and contents
    write_entry_header(writer, &path, file_size)?;
    writer.write_all(&contents)?;

    Ok(())
}

/// Writes a .knytt.bin entry header to `writer`.
fn write_entry_header(writer: &mut BufWriter<File>, name: &str, len: usize) -> Result<()> {
    let len: u32 = len
        .try_into()
        .expect("Entry length should not exceed u32::MAX bytes");

    writer.write_all(&ENTRY_SIGNATURE)?;
    writer.write_all(name.as_bytes())?;
    writer.write_all(&[0u8])?; // null terminator
    writer.write_u32::<LittleEndian>(len)?;

    Ok(())
}

/// Converts the name of the current working directory to a `String`.
fn name_of_current_dir() -> Result<String> {
    let current_dir = env::current_dir()?;
    if let Some(name) = current_dir.file_name().and_then(|s| s.to_str()) {
        Ok(name.to_owned())
    }
    else {
        Err(KnyttBinError::BadFileName(current_dir).into())
    }
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
