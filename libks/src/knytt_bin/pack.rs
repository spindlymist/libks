use std::{
    env,
    fs::{self, File, OpenOptions},
    path::Path,
    io::{BufWriter, Write, SeekFrom, Seek},
};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::Result;
use super::{KnyttBinError, ENTRY_SIGNATURE};

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
