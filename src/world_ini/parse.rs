use std::collections::HashMap;

use ini::{Ini, Properties};

use crate::{
    map_bin::AssetId,
    world_ini::model::*,
};

pub fn parse_ini(ini: &Ini) -> WorldIni {
    WorldIni {
        world: parse_world_section(ini),
        cutscene_music: parse_cutscene_music_section(ini),
        loop_music: parse_loop_music_section(ini),
        cutscene_color: parse_cutscene_color_section(ini),
        custom_characters: parse_custom_character_section(ini),
        custom_objects: parse_custom_objects_sections(ini, false),
        custom_objects_b: parse_custom_objects_sections(ini, true),
        screens: parse_screens_section(ini)
    }
}

pub fn parse_world_section(_ini: &Ini) -> WorldSection {
    WorldSection { ..Default::default() }
}

pub fn parse_cutscene_music_section(_ini: &Ini) -> CutsceneMusicSection {
    CutsceneMusicSection { ..Default::default() }
}

pub fn parse_loop_music_section(_ini: &Ini) -> LoopMusicSection {
    LoopMusicSection { ..Default::default() }
}

pub fn parse_cutscene_color_section(_ini: &Ini) -> CutsceneColorSection {
    CutsceneColorSection { ..Default::default() }
}

pub fn parse_custom_character_section(_ini: &Ini) -> CustomCharacterSection {
    CustomCharacterSection { ..Default::default() }
}

pub fn parse_custom_objects_sections(ini: &Ini, b_bank: bool) -> HashMap<AssetId, CustomObjectSection> {
    let make_key = match b_bank {
        false => |index| Some(format!("Custom Object {index}")),
        true => |index| Some(format!("Custom Object B{index}")),
    };
    let mut sections = HashMap::new();

    for i in 0u8..255u8 {
        if let Some(props) = ini.section(make_key(i)) {
            sections.insert(i, parse_custom_object_section(props));
        }
    }

    sections
}

pub fn parse_custom_object_section(props: &Properties) -> CustomObjectSection {
    let image = props.get("Image").map(|val| val.to_owned());
    let size = (
        props.get("Tile Width")
            .and_then(|val| str::parse::<u32>(val).ok())
            .unwrap_or(24),
        props.get("Tile Height")
            .and_then(|val| str::parse::<u32>(val).ok())
            .unwrap_or(24),
    );
    let offset = (
        props.get("Offset X")
            .and_then(|val| str::parse::<i32>(val).ok())
            .unwrap_or(24),
        props.get("Offset Y")
            .and_then(|val| str::parse::<i32>(val).ok())
            .unwrap_or(24),
    );
    let animation = parse_animation_params(props);
    let override_object = None;
    let is_harmless = props.get("Hurts") == Some("False");
    let color = None;

    CustomObjectSection {
        image,
        size,
        offset,
        animation,
        override_object,
        is_harmless,
        color,
    }
}

pub fn parse_animation_params(props: &Properties) -> AnimationParams {
    let anim_from = props.get("Init AnimFrom")
        .and_then(|val| str::parse::<u32>(val).ok())
        .unwrap_or(0);
    let anim_to = props.get("Init AnimTo")
        .and_then(|val| str::parse::<u32>(val).ok())
        .unwrap_or(0);
    let anim_loop_back = props.get("Init AnimLoopBack")
        .and_then(|val| str::parse::<u32>(val).ok())
        .unwrap_or(0);
    let anim_repeat = props.get("Init AnimRepeat")
        .and_then(|val| str::parse::<u32>(val).ok())
        .unwrap_or(0);
    
    AnimationParams {
        anim_from,
        anim_to,
        anim_loop_back,
        anim_repeat,
    }
}

pub fn parse_screens_section(_ini: &Ini) -> HashMap<(i64, i64), ScreenSection> {
    HashMap::new()
}
