use std::{path::{Path, PathBuf}};

use crate::Result;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum KsEdition {
    Vanilla,
    Plus,
    Extended,
    Advanced,
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

/// Returns `true` if the directory at `path` appears contain a Knytt Stories installation.
/// 
/// In particular, the directory must contain a Worlds folder, a Data folder, and one or more
/// KS executables.
pub fn is_ks_dir(path: &Path) -> bool {
    path.is_dir()
        && path.join("Worlds").exists()
        && path.join("Data").exists()
        && !list_executables(path).is_empty()
}

/// Determines which KS executables are present in `ks_dir`.
pub fn list_executables(ks_dir: &Path) -> Vec<KsExecutable> {
    use KsEdition::*;
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
/// The heuristic is not comprehensive and this function cannot currently detect KS Advanced levels.
pub fn guess_edition(world_dir: &Path) -> Result<KsEdition> {
    use KsEdition::*;

    let world_ini = crate::world_ini::load_ini(world_dir)?;
    if world_ini.section(Some("World"))
        .and_then(|section| section.get("Format"))
        == Some("4")
    {
        return Ok(Plus);
    }

    if world_dir.join("Script.lua").exists()
        || world_ini.section(Some("KS Ex")).is_some()
        || world_ini.section(Some("Templates")).is_some()
    {
        return Ok(Extended);
    }

    Ok(KsEdition::default())
}
