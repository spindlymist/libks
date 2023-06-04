use std::{collections::HashSet, path::Path, process::Command, ffi::OsStr};

use crate::Result;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum KsEdition {
    Vanilla,
    Plus,
    Extended,
    Advanced,
}

impl KsEdition {
    /// Returns the primary executable associated with this edition.
    pub fn exe_name(&self) -> &'static str {
        use KsEdition::*;
        match self {
            Vanilla => "Knytt Stories.exe",
            Plus => "Knytt Stories Plus.exe",
            Extended => "Knytt Stories Ex.exe",
            Advanced => "KSAdvanced.exe",
        }
    }
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
        && !list_editions(path).is_empty()
}

/// Determines which KS editions are installed in `ks_dir`.
/// 
/// Note that this function doesn't verify the integrity of the installation,
/// just the presence of the primary executable.
pub fn list_editions(ks_dir: &Path) -> HashSet<KsEdition> {
    use KsEdition::*;
    let mut editions = HashSet::new();

    for edition in [Vanilla, Plus, Extended, Advanced] {
        if ks_dir.join(edition.exe_name()).exists() {
            editions.insert(edition);
        }
    }

    editions
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

/// Launches the primary executable for the chosen edition (default: vanilla).
pub fn launch_ks(ks_dir: &Path, edition: Option<KsEdition>) -> Result<()> {
    let edition = edition.unwrap_or_default();
    let exe_path = ks_dir.join(edition.exe_name());
    Command::new(exe_path)
        .spawn()?;

    Ok(())
}

/// Launches the primary executable for the chosen edition (default: vanilla) with the specified arguments.
pub fn launch_ks_with_args<I, S>(ks_dir: &Path, edition: Option<KsEdition>, args: I) -> Result<()>
where
    I: Iterator<Item = S>,
    S: AsRef<OsStr>,
{
    let edition = edition.unwrap_or_default();
    let exe_path = ks_dir.join(edition.exe_name());
    Command::new(exe_path)
        .args(args)
        .spawn()?;

    Ok(())
}
