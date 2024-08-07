use std::{
    ops::RangeBounds,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{editions::small_set::static_set_lowercase, Result};
use super::KsEdition;

#[allow(clippy::enum_variant_names)]
pub enum FilesReason {
    HasScriptDotLua,
    HasInfoPlus,
    HasKsAdvancedScene(PathBuf),
    HasKsPlusIconOverride(PathBuf),
    HasKsPlusSongIntro(PathBuf),
}

impl std::fmt::Display for FilesReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilesReason::HasScriptDotLua =>
                write!(f, "Script.lua is present in the level directory."),
            FilesReason::HasInfoPlus =>
                write!(f, "Icon+.png is present in the level directory."),
            FilesReason::HasKsAdvancedScene(path) =>
                write!(f, "A KS Advanced scene definition exists at this path: `{}`", path.to_string_lossy()),
            FilesReason::HasKsPlusIconOverride(path) =>
                write!(f, "A KS Plus icon override exists at this path: `{}`", path.to_string_lossy()),
            FilesReason::HasKsPlusSongIntro(path) =>
                write!(f, "A KS Plus song intro exists at this path: `{}`", path.to_string_lossy()),
        }
    }
}

pub fn check_files_basic(world_dir: &Path) -> Result<Option<(KsEdition, FilesReason)>> {
    use KsEdition::*;
    use FilesReason::*;

    if world_dir.join("Script.lua").try_exists()? {
        return Ok(Some((Extended, HasScriptDotLua)));
    }

    if world_dir.join("Info+.png").try_exists()? {
        return Ok(Some((Plus, HasInfoPlus)));
    }

    Ok(None)
}

/// KS+
///   - Custom Objects/PowerIcon{0-12}.png
///   - Custom Objects/CoinIcon.png
///   - Custom Objects/ArtifactIcon.png
///   - Custom Objects/CreatureIcon.png
///   - Music/Intro#.ogg
/// KSA
///   - */Scene#.ini excluding vanilla directories
pub fn check_files_thorough(world_dir: &Path) -> Result<Option<(KsEdition, FilesReason)>> {
    use KsEdition::*;
    use FilesReason::*;

    let plus_icon_overrides = static_set_lowercase![
        "CoinIcon.png",
        "ArtifactIcon.png",
        "CreatureIcon.png",
        "PowerIcon0.png",
        "PowerIcon1.png",
        "PowerIcon2.png",
        "PowerIcon3.png",
        "PowerIcon4.png",
        "PowerIcon5.png",
        "PowerIcon6.png",
        "PowerIcon7.png",
        "PowerIcon8.png",
        "PowerIcon9.png",
        "PowerIcon10.png",
        "PowerIcon11.png",
        "PowerIcon12.png",
    ];
    let vanilla_directories = static_set_lowercase![
        "Ambiance",
        "Custom Objects",
        "Gradients",
        "Music",
        "Tilesets",
    ];

    let is_adv_scene_definition = |name: &str| {
        let name = name.to_ascii_lowercase();
        is_range_with_affixes(&name, "scene", ".ini", 1..)
    };
    let is_plus_icon_override = |name: &str| {
        let name = name.to_ascii_lowercase();
        plus_icon_overrides.has(&name.as_str())
    };
    let is_plus_song_intro = |name: &str| {
        let name = name.to_ascii_lowercase();
        is_range_with_affixes(&name, "intro", ".ogg", 1..=255)
    };

    // Check for KS Advanced scene definitions
    for entry in world_dir.read_dir()? {
        let entry = entry?;

        let file_type = entry.file_type()?;
        if !file_type.is_dir() { continue; }

        let dir_name = entry.file_name();
        let Some(dir_name) = dir_name.to_str() else {
            continue;
        };

        let dir_name_lower = dir_name.to_ascii_lowercase();
        if vanilla_directories.has(&dir_name_lower.as_str()) {
            continue;
        }

        if let Some(file_name) = find_in_directory(&entry.path(), is_adv_scene_definition)? {
            let path: PathBuf = [dir_name, &file_name].iter().collect();
            let reason = HasKsAdvancedScene(path);
            return Ok(Some((Advanced, reason)));
        }
    }

    // Check for KS Plus icon overrides
    let custom_objects_dir = world_dir.join("Custom Objects");
    if let Some(file_name) = find_in_directory(&custom_objects_dir, is_plus_icon_override)? {
        let path: PathBuf = ["Custom Objects", &file_name].iter().collect();
        let reason = HasKsPlusIconOverride(path);
        return Ok(Some((Plus, reason)));
    }

    // Check for KS Plus song intros
    let music_dir = world_dir.join("Music");
    if let Some(file_name) = find_in_directory(&music_dir, is_plus_song_intro)? {
        let path: PathBuf = ["Music", &file_name].iter().collect();
        let reason = HasKsPlusSongIntro(path);
        return Ok(Some((Plus, reason)));
    }

    Ok(None)
}

fn find_in_directory<F>(dir: &Path, mut predicate: F) -> Result<Option<String>>
where
    F: FnMut(&str) -> bool,
{
    match dir.metadata() {
        Ok(meta) if !meta.is_dir() => return Ok(None),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.into()),
        _ => (),
    };

    for entry in dir.read_dir()? {
        let file_name = entry?.file_name();
        if let Some(file_name) = file_name.to_str() {
            if predicate(file_name) {
                return Ok(Some(file_name.to_owned()));
            }
        }
    }

    Ok(None)
}

fn is_range_with_affixes<B, T>(s: &str, prefix: &str, suffix: &str, range: B) -> bool
where
    B: RangeBounds<T>,
    T: FromStr + PartialOrd<T> + ?Sized,
{
    let Some(affixed) = s.strip_suffix(suffix)
        .and_then(|s| s.strip_prefix(prefix))
    else {
        return false;
    };

    let Ok(number) = str::parse::<T>(affixed) else {
        return false;
    };

    range.contains(&number)
}
