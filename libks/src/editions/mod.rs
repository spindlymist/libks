use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use crate::{Result, map_bin, world_ini};

mod file_system_heuristics;
use file_system_heuristics::{check_files_basic, check_files_thorough, FilesReason};
mod map_bin_heuristics;
use map_bin_heuristics::{check_map_bin, MapBinReason};
mod world_ini_heuristics;
use world_ini_heuristics::{
    check_ini_basic,
    check_ini_format,
    check_ini_thorough,
    IniReason,
};

mod small_set;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum KsEdition {
    Vanilla,
    Plus,
    Extended,
    Advanced,
    AdvancedCustomObjects,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct KsExecutable {
    pub edition: KsEdition,
    pub path: PathBuf,
}

impl Default for KsEdition {
    fn default() -> Self {
        Self::Vanilla
    }
}

pub enum Reason {
    Ini(IniReason),
    Files(FilesReason),
    MapBin(MapBinReason),
    Default,
}

impl From<IniReason> for Reason {
    fn from(reason: IniReason) -> Self {
        Self::Ini(reason)
    }
}

impl From<FilesReason> for Reason {
    fn from(reason: FilesReason) -> Self {
        Self::Files(reason)
    }
}

impl From<MapBinReason> for Reason {
    fn from(reason: MapBinReason) -> Self {
        Self::MapBin(reason)
    }
}

impl Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Reason::*;
        match self {
            Ini(reason) => f.write_str(&reason.to_string()),
            Files(reason) => f.write_str(&reason.to_string()),
            MapBin(reason) => f.write_str(&reason.to_string()),
            Default => write!(f, "No features from any mods were detected."),
        }
    }
}

/// Returns `true` if the directory at `path` appears contain a Knytt Stories installation.
/// 
/// In particular, the directory must contain a Worlds folder, a Data folder, and one or more
/// KS executables.
pub fn is_ks_dir<P>(path: P) -> bool
where
    P: AsRef<Path>
{
    let path = path.as_ref();
    path.is_dir()
        && path.join("Worlds").exists()
        && path.join("Data").exists()
        && !list_executables(path).is_empty()
}

/// Determines which KS executables are present in `ks_dir`.
pub fn list_executables<P>(ks_dir: P) -> Vec<KsExecutable>
where
    P: AsRef<Path>
{
    use KsEdition::*;
    let ks_dir = ks_dir.as_ref();
    let mut exes = Vec::new();

    for (edition, exe_name) in [
        (Vanilla, "Knytt Stories.exe"),
        (Plus, "Knytt Stories Plus.exe"),
        (Plus, "Knytt Stories Plus 1080.exe"),
        (Extended, "Knytt Stories Ex.exe"),
        (Advanced, "KSAdvanced.exe"),
    ] {
        let path = ks_dir.join(exe_name);
        if path.exists() {
            exes.push(KsExecutable {
                edition,
                path
            });
        }
    }

    exes
}

/// Attempts to determine what KS edition the level in `world_dir` is made for. Defaults to
/// vanilla.
/// 
/// This variant prioritizes speed over accuracy. The heuristic used is not comprehensive,
/// and it can only detect KS Plus and KS Extended levels.
/// 
/// See also: [guess_edition_accurate]
pub fn guess_edition_fast<P>(world_dir: P) -> Result<(KsEdition, Reason)>
where
    P: AsRef<Path>
{
    let world_dir = world_dir.as_ref();
    let world_ini = world_ini::load_ini(world_dir)?;
    
    if let Some((edition, reason)) = check_ini_format(&world_ini) {
        return Ok((edition, reason.into()));
    }

    if let Some((edition, reason)) = check_files_basic(world_dir)? {
        return Ok((edition, reason.into()));
    }

    if let Some((edition, reason)) = check_ini_basic(&world_ini) {
        return Ok((edition, reason.into()));
    }

    Ok((KsEdition::default(), Reason::Default))
}

/// Attempts to determine what KS edition the level in `world_dir` is made for. Defaults to
/// vanilla.
/// 
/// This variant prioritizes accuracy over speed, but it is not infallible. While it can detect
/// all editions, the more obscure ones are less likely to be recognized. Ironically, vanilla
/// levels take the longest to detect because every other edition's minor features have to be
/// ruled out.
/// 
/// See also: [guess_edition_fast]
pub fn guess_edition_accurate<P>(world_dir: P) -> Result<(KsEdition, Reason)>
where
    P: AsRef<Path>,
{
    let world_dir = world_dir.as_ref();
    let world_ini = world_ini::load_ini(world_dir)?;
    
    if let Some((edition, reason)) = check_ini_format(&world_ini) {
        return Ok((edition, reason.into()));
    }

    if let Some((edition, reason)) = check_files_basic(world_dir)? {
        return Ok((edition, reason.into()));
    }

    if let Some((edition, reason)) = check_ini_basic(&world_ini) {
        return Ok((edition, reason.into()));
    }

    if let Some((edition, reason)) = check_ini_thorough(&world_ini) {
        return Ok((edition, reason.into()));
    }

    if let Some((edition, reason)) = check_files_thorough(world_dir)? {
        return Ok((edition, reason.into()));
    }

    let screens = map_bin::parse_map_file(world_dir.join("Map.bin"))?;
    if let Some((edition, reason)) = check_map_bin(&screens) {
        return Ok((edition, reason.into()));
    }

    Ok((KsEdition::default(), Reason::Default))
}
