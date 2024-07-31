use std::collections::HashMap;

use crate::map_bin::{AssetId, Tile};

#[derive(Default)]
pub struct WorldIni {
    pub world: WorldSection,
    pub cutscene_music: CutsceneMusicSection,
    pub loop_music: LoopMusicSection,
    pub cutscene_color: CutsceneColorSection,
    pub custom_characters: CustomCharacterSection,
    pub custom_objects: HashMap<AssetId, CustomObjectSection>,
    pub custom_objects_b: HashMap<AssetId, CustomObjectSection>,
    pub screens: HashMap<(i64, i64), ScreenSection>,
}

#[derive(Default)]
pub struct WorldSection {
    pub name: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub categories: [Option<Category>; 2],
    pub difficulties: [Option<Difficulty>; 3],
    pub size: Option<Size>,
    pub format: Option<Format>,
    pub clothes: Option<Color>,
    pub skin: Option<Color>,

    // KS+
    pub holo_fix: Option<bool>,
    pub holo_limit: Option<bool>,
    pub new: Option<bool>,
    pub map: Option<bool>,
    pub ambi_fade: [Option<bool>; 2],
    pub fonts: FontParams,
    pub character: Option<String>,
    pub powers: Option<String>,
    pub coin: Option<String>,
    pub artifacts: [Option<String>; 7],
}

#[derive(Default)]
pub struct FontParams {
    pub font: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
}

#[derive(Default)]
pub struct CutsceneMusicSection {
    pub cutscenes: HashMap<String, AssetId>,
}

#[derive(Default)]
pub struct LoopMusicSection {
}

#[derive(Default)]
pub struct CutsceneColorSection {
}

#[derive(Default)]
pub struct CustomCharacterSection {
}

#[derive(Default)]
pub struct CustomObjectSection {
    pub image: Option<String>,
    pub size: (u32, u32),
    pub offset: (i32, i32),
    pub animation: AnimationParams,
    pub override_object: Option<Tile>,
    pub is_harmless: bool,
    pub color: Option<Color>,
}

#[derive(Default)]
pub struct AnimationParams {
    pub anim_from: u32,
    pub anim_to: u32,
    pub anim_loop_back: u32,
    pub anim_repeat: u32,
}

#[derive(Default)]
pub struct ScreenSection {
    pub signs: [Option<SignParams>; 3],
    pub wraps: [Option<WarpParams>; 3],
    pub shifts: [Option<ShiftParams>; 3],
    pub triggers: [Option<TriggerParams>; 3],
    pub ending: Option<String>,
    pub map: Option<MapParams>,
    pub tint: Option<TintParams>,
    pub attachment: Option<String>,
    pub overlay: Option<bool>,
}

#[derive(Default)]
pub struct SignParams {
}

#[derive(Default)]
pub struct WarpParams {
}

#[derive(Default)]
pub struct ShiftParams {
    pub absolute_target: bool,
    pub invisible: bool,
    pub touch: bool,
    pub quantize: bool,
    pub autosave: bool,
    pub stop_music: bool,
    pub show_effect: bool,
    pub deny_hologram: bool,
    pub hide: bool,
    pub delay: u32,
    pub coin: u8,
    pub map: (i64, i64),
    pub position: (i64, i64),
    pub shift_type: Option<ShiftType>,
    pub sound: Option<ShiftSound>,
    pub cutscene: String,
    pub flag_on: Option<Flag>,
    pub flag_off: Option<Flag>,
    pub character: String,
}

pub enum Flag {
}

#[derive(Default)]
pub struct TriggerParams {
    pub absolute_target: bool,
    pub invisible: bool,
    pub touch: bool,
    pub as_one: bool,
    pub repeatable: bool,
    pub show_effect: bool,
    pub deny_hologram: bool,
    pub object: Option<Tile>,
    pub spawn: (i64, i64),
    pub effect_offset: (i64, i64),
    pub trigger_type: Option<ShiftType>,
    pub sound: Option<ShiftSound>,
}

pub enum ShiftType {
    Spot,
    Floor,
    Circle,
    Square,
}

pub enum ShiftSound {
    None,
    Default,
    Switch,
    Door,
    Electronic,
    Custom(String),
}

#[derive(Default)]
pub struct MapParams {
    pub visible: bool,
    pub color: Color,
    pub position: (i64, i64),
}

#[derive(Default)]
pub struct TintParams {
    pub transparency: Option<u8>,
    pub ink: Option<TintInk>,
    pub color: Option<Color>,
}

pub enum TintInk {
    Trans,
    Add,
    Sub,
    AND,
    OR,
    XOR,
}

#[derive(Default)]
pub struct Color(pub u8, pub u8, pub u8);

pub enum Category {
    Tutorial,
    Challenge,
    Puzzle,
    Maze,
    Environmental,
    Playground,
    Misc,
    Unknown(String),
}

pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    VeryHard,
    Lunatic,
    Unknown(String),
}

pub enum Size {
    Small,
    Medium,
    Large,
    Unknown(String),
}

pub enum Format {
    Vanilla1,
    Vanilla2,
    KsPlus,
    KsEx,
    Unknown(String),
}
