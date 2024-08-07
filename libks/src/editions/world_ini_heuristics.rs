use std::{
    collections::HashSet, ops::RangeBounds, str::FromStr
};

use libks_ini::Ini;

use crate::common::parse_xy;
use super::{
    small_set::{static_set_lowercase_from_file, static_set_lowercase},
    KsEdition,
};

pub enum IniReason {
    HasFormat(String),
    HasSection(String),
    WorldSectionHasProp(String),
    ObjectSectionHasProp(String, String),
    ScreenSectionHasProp(String, String),
    ScreenSectionHasCoinFlag(String),
    ScreenSectionHasArtifactWarp(String),
    HasKsAdvancedProps(usize, Vec<String>),
    HasKsACOProps(usize, Vec<String>),
}

impl std::fmt::Display for IniReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use IniReason::*;

        match self {
            HasFormat(format) =>
                write!(f, "World.ini specifies Format = `{format}`."),
            HasSection(key) =>
                write!(f, "World.ini contains the section [{key}]."),
            WorldSectionHasProp(key) =>
                write!(f, "The [World] section of World.ini has the property `{key}`."),
            ObjectSectionHasProp(section_key, prop_key) =>
                write!(f, "In World.ini, the object section [${section_key}] has the property `{prop_key}`."),
            ScreenSectionHasProp(section_key, prop_key) =>
                write!(f, "In World.ini, the screen section [${section_key}] has the property `{prop_key}`."),
            ScreenSectionHasCoinFlag(key) =>
                write!(f, "In World.ini, the screen section [${key}] has a coin flag."),
            ScreenSectionHasArtifactWarp(key) =>
                write!(f, "In World.ini, the screen section [${key}] has an artifact warp."),
            HasKsAdvancedProps(count, keys) =>
                write!(f, "World.ini uses these KS Advanced properties {count} time(s): `{}`", keys.join("`, `")),
            HasKsACOProps(count, keys) =>
                write!(f, "World.ini uses these KS ACO properties {count} time(s): `{}`", keys.join("`, `")),
        }
    }
}

pub fn check_ini_format(world_ini: &Ini) -> Option<(KsEdition, IniReason)> {
    use KsEdition::*;
    use IniReason::*;

    let world = world_ini.section("World")?;

    match world.get("Format") {
        Some("4") => {
            let reason = HasFormat("4".to_owned()); 
            return Some((Plus, reason));
        },
        Some("3") => {
            let reason = HasFormat("3".to_owned()); 
            return Some((Extended, reason));
        },
        _ => (),
    };

    if world.has("FormatEx") {
        let reason = WorldSectionHasProp("FormatEx".to_owned());
        return Some((Extended, reason));
    }

    None
}

pub fn check_ini_basic(world_ini: &Ini) -> Option<(KsEdition, IniReason)> {
    use KsEdition::*;
    use IniReason::*;

    // Check for KS Ex sections
    for section_key in ["KS Ex", "Templates"] {
        if world_ini.has_section(section_key) {
            let reason = HasSection(section_key.to_owned());
            return Some((Extended, reason));
        }
    }

    // Check for KS Plus sections
    for section_key in ["Loop Music", "Cutscene Color", "Custom Character"] {
        if world_ini.has_section(section_key) {
            let reason = HasSection(section_key.to_owned());
            return Some((Plus, reason));
        }
    }
    
    // Check for KS Plus world properties
    let world = world_ini.section("World")?;
    let plus_world_props = static_set_lowercase![
        "HoloFix",
        "Character",
        "Map",
        "Font",
        "Sign",
        "Title",
        "Subtitle",
        "Powers",
        "Coin",
        "Artifact1",
        "Artifact2",
        "Artifact3",
        "Artifact4",
        "Artifact5",
        "Artifact6",
        "Artifact7",
        "SinglePass",
        "AltDie",
    ];

    if let Some((key, _)) = world.iter().find(|(key, _)| plus_world_props.has(key)) {
        let reason = WorldSectionHasProp(key.to_owned());
        return Some((Plus, reason));
    }

    // Check for KS Advanced world properties
    if world.has("DeathByFalling") {
        let reason = WorldSectionHasProp("DeathByFalling".into());
        return Some((Advanced, reason));
    }

    None
}

/// KS+
///   - COs:
///     - Custom Object B#
///     - Bank, Object, Hurts, Color
///   - Screens:
///     - Trigger properties
///     - Extended Shift properties
///     - Extended Sign properties
///     - Title/subtitle
///     - Overlay, Tint, TintTrans, TintInk, Attach
///     - Map properties
///     - Artifact warps
///     - Coin flags
/// KS Ex
///   - Screens: Signs can have custom labels besides A, B, and C. However, they only work with a Script.lua
/// KSA
///   - World: DeathByFalling
///   - Screen:
///     - ChangeToColor
///     - Replace(R)
///     - Replace(G)
///     - Replace(B)
/// KS ACO
///   - COs:
///     - Does kill
///     - Type
///   - Screens: WarpSave
pub fn check_ini_thorough(world_ini: &Ini) -> Option<(KsEdition, IniReason)> {
    use KsEdition::*;
    use IniReason::*;

    let flag_props = static_set_lowercase!["Flag(A)", "Flag(B)", "Flag(C)"];
    let flag_warp_props = static_set_lowercase![
        "FlagWarpX(A)", "FlagWarpX(B)", "FlagWarpX(C)",
        "FlagWarpY(A)", "FlagWarpY(B)", "FlagWarpY(C)",
    ];
    let plus_object_props = static_set_lowercase!["Bank", "Object", "Hurts", "Color"];
    let plus_screen_props = static_set_lowercase_from_file!("data/plus_screen_props.txt");
    let adv_screen_props = static_set_lowercase![
        "ChangeToColor", "Replace(R)", "Replace(G)", "Replace(B)",
    ];
    let aco_object_props = static_set_lowercase!["Does kill", "Type"];
    let aco_screen_props = static_set_lowercase!["WarpSave"];

    let is_object_section = |key: &str| {
        // Expects lowercase key
        is_range_with_prefix(key, "custom object", 1..=255)
    };
    let is_screen_section = |key: &str| {
        // Expects lowercase key
        parse_xy(key).is_some()
    };
    let is_plus_b_bank_object_section = |key: &str| {
        is_range_with_prefix(key, "custom object b", 1..=255)
    };
    let is_plus_coin_flag = |value: &str| {
        is_range_with_prefix(&value.to_ascii_lowercase(), "coin", 1..=100)
    };
    let is_plus_artifact_warp = |value: &str| {
        is_range_with_prefix(&value.to_ascii_lowercase(), "artifact", 1..=7)
    };

    let mut adv_seen = HashSet::new();
    let mut adv_count = 0;

    let mut aco_seen = HashSet::new();
    let mut aco_count = 0;

    for section in world_ini.iter_sections() {
        let section_key = section.key();
        let section_key_lower = section_key.to_ascii_lowercase();

        if is_plus_b_bank_object_section(&section_key_lower) {
            let reason = HasSection(section_key.to_owned());
            return Some((Plus, reason));
        }
        else if is_object_section(&section_key_lower) {
            for (key, _) in section.iter() {
                let lower_key = key.to_ascii_lowercase();
                let lower_key = lower_key.as_str();

                if plus_object_props.has(&lower_key) {
                    let reason = ObjectSectionHasProp(section_key.to_owned(), key.to_owned());
                    return Some((Plus, reason));
                }
                else if aco_object_props.has(&lower_key) {
                    aco_count += 1;
                    aco_seen.insert(key);
                }
            }
        }
        else if is_screen_section(&section_key_lower) {
            for (key, value) in section.iter() {
                let lower_key = key.to_ascii_lowercase();
                let lower_key = lower_key.as_str();

                if plus_screen_props.has(&lower_key) {
                    let reason = ScreenSectionHasProp(section_key.to_owned(), key.to_owned());
                    return Some((Plus, reason));
                }
                else if flag_props.has(&lower_key)
                    && is_plus_coin_flag(value)
                {
                    let reason = ScreenSectionHasCoinFlag(section_key.to_owned());
                    return Some((Plus, reason));
                }
                else if flag_warp_props.has(&lower_key)
                    && is_plus_artifact_warp(value)
                {
                    let reason = ScreenSectionHasArtifactWarp(section_key.to_owned());
                    return Some((Plus, reason));
                }
                else if adv_screen_props.has(&lower_key) {
                    adv_count += 1;
                    adv_seen.insert(key);
                }
                else if aco_screen_props.has(&lower_key) {
                    aco_count += 1;
                    aco_seen.insert(key);
                }
            }
        }
    }
    
    if adv_count > aco_count {
        let props: Vec<String> = adv_seen.into_iter()
            .map(|key| key.to_owned())
            .collect();
        let reason = HasKsAdvancedProps(adv_count, props);
        Some((Advanced, reason))
    }
    else if aco_count > 0 {
        let props: Vec<String> = aco_seen.into_iter()
            .map(|key| key.to_owned())
            .collect();
        let reason = HasKsACOProps(aco_count, props);
        Some((AdvancedCustomObjects, reason))
    }
    else {
        None
    }
}

fn is_range_with_prefix<B, T>(s: &str, prefix: &str, range: B) -> bool
where
    B: RangeBounds<T>,
    T: FromStr + PartialOrd<T> + ?Sized,
{
    let Some(suffix) = s.strip_prefix(prefix) else {
        return false;
    };

    let Ok(number) = str::parse::<T>(suffix) else {
        return false;
    };

    range.contains(&number)
}
