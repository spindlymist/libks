use std::collections::HashSet;

use crate::map_bin::{ScreenData, Tile};
use super::KsEdition;

#[allow(clippy::enum_variant_names)]
pub enum MapBinReason {
    HasKsPlusObject(Tile),
    HasKsAdvancedObjects(usize, Vec<Tile>),
    HasKsACOObjects(usize, Vec<Tile>),
}

impl std::fmt::Display for MapBinReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tiles_to_string = |tiles: &[Tile]| {
            let strings: Vec<_> = tiles.iter()
                .map(|tile| format!("{}:{}", tile.0, tile.1))
                .collect();
            strings.join(", ")
        };

        match self {
            MapBinReason::HasKsPlusObject(tile) =>
                write!(f, "Map.bin uses the KS Plus object {}:{}.", tile.0, tile.1),
            MapBinReason::HasKsAdvancedObjects(count, tiles) =>
                write!(f, "Map.bin uses these KS Advanced objects {count} time(s): {}", &tiles_to_string(tiles)),
            MapBinReason::HasKsACOObjects(count, tiles) =>
                write!(f, "Map.bin uses these KS ACO objects {count} time(s): {}", &tiles_to_string(tiles)),
        }
    }
}

/// KS+:
///   - Bank 0   32-49 (32 overlaps with KS Ex)
///   - Bank 0   247-255
///   - Bank 1   25-27
///   - Bank 6   14-17
///   - Bank 7   17
///   - Bank 15  31-38
///   - Bank 16  17-30
///   - Bank 19  1-199
///   - Bank 254 if combined with [Custom Object B#]
/// KS Ex:
///   - Bank 0   32 (overlaps with KS+)
///   - Bank 254 if combined with [Templates]
/// KSA:
///   - Bank 254 1-22 if no [Custom Object B#] or [Templates]
/// KS ACO:
///   - Bank 253 1-6
///   - Bank 254 1-3 prior to 1.2.0
pub fn check_map_bin(screens: &[ScreenData]) -> Option<(KsEdition, MapBinReason)> {
    use KsEdition::*;
    use MapBinReason::*;

    let is_plus_object = |tile: &Tile| {
        matches!(tile,
            Tile(0, 33..=49)
            | Tile(0, 247..=255)
            | Tile(1, 25..=27)
            | Tile(6, 14..=17)
            | Tile(7, 17)
            | Tile(15, 31..=38)
            | Tile(16, 17..=30)
            | Tile(19, 1..=199)
        )
    };
    let is_adv_object = |Tile(bank, idx): &Tile| {
        *bank == 254 && *idx <= 22
    };
    let is_aco_object = |Tile(bank, idx): &Tile| {
        *bank == 253 && *idx <= 6
    };

    let mut adv_seen = HashSet::new();
    let mut adv_count = 0;

    let mut aco_seen = HashSet::new();
    let mut aco_count = 0;
    
    for screen in screens {
        for layer in screen.layers.iter().skip(4) {
            for tile in &layer.0 {
                if tile.1 == 0 {
                    continue;
                }
                else if is_plus_object(tile) {
                    let reason = HasKsPlusObject(*tile);
                    return Some((Plus, reason));
                }
                else if is_adv_object(tile) {
                    adv_count += 1;
                    adv_seen.insert(*tile);
                }
                else if is_aco_object(tile) {
                    aco_count += 1;
                    aco_seen.insert(*tile);
                }
            }
        }
    }

    if adv_count > aco_count {
        let tiles: Vec<Tile> = adv_seen.into_iter().collect();
        let reason = HasKsAdvancedObjects(adv_count, tiles);
        Some((Advanced, reason))
    }
    else if aco_count > 0 {
        let tiles: Vec<Tile> = aco_seen.into_iter().collect();
        let reason = HasKsACOObjects(aco_count, tiles);
        Some((AdvancedCustomObjects, reason))
    }
    else {
        None
    }
}
