//! Global Blackboard: Variables, Character Entity Registry, Audio Assets & Scene Flow Outline
use eframe::egui::Color32;

/// Supported variable data types in the narrative blackboard.
#[derive(Clone, Debug, PartialEq)]
pub enum VariableType {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

impl VariableType {
    pub fn type_name(&self) -> &'static str {
        match self {
            VariableType::Boolean(_) => "Bool",
            VariableType::Integer(_) => "Int",
            VariableType::Float(_) => "Float",
            VariableType::String(_) => "String",
        }
    }
}

/// A global game-state variable definition.
#[derive(Clone, Debug, PartialEq)]
pub struct BlackboardVariable {
    pub name: String,
    pub var_type: VariableType,
    pub description: String,
}

/// Character/Speaker metadata for dialogue attribution and UI color swatches.
#[derive(Clone, Debug, PartialEq)]
pub struct CharacterEntity {
    pub id: String,
    pub name: String,
    pub color: Color32,
    pub portrait_path: String,
}

/// Sound cue entry binding dialogue lines to external audio files.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioEntry {
    pub key: String,
    pub filename: String,
    pub speaker: String,
    pub duration_sec: f32,
}

/// High-level narrative tree structure item.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneOutlineItem {
    pub id: usize,
    pub title: String,
    pub chapter: String,
}

/// The centralized repository of game variables, speakers, and audio assets.
#[derive(Clone, Debug, PartialEq)]
pub struct Blackboard {
    pub variables: Vec<BlackboardVariable>,
    pub characters: Vec<CharacterEntity>,
    pub audio_assets: Vec<AudioEntry>,
    pub scene_outline: Vec<SceneOutlineItem>,
}

impl Default for Blackboard {
    fn default() -> Self {
        Self::default_demo()
    }
}

impl Blackboard {
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
            characters: Vec::new(),
            audio_assets: Vec::new(),
            scene_outline: Vec::new(),
        }
    }

    /// Pre-loaded demo blackboard matching the specification and reference assets.
    pub fn default_demo() -> Self {
        Self {
            variables: vec![
                BlackboardVariable {
                    name: "has_sword".to_string(),
                    var_type: VariableType::Boolean(true),
                    description: "Player acquired the royal pen-blade".to_string(),
                },
                BlackboardVariable {
                    name: "reputation".to_string(),
                    var_type: VariableType::Integer(75),
                    description: "Faction standing with the Royal Court".to_string(),
                },
                BlackboardVariable {
                    name: "quest_completed".to_string(),
                    var_type: VariableType::Boolean(false),
                    description: "Main gate investigation resolved".to_string(),
                },
            ],
            characters: vec![
                CharacterEntity {
                    id: "king_aurther".to_string(),
                    name: "King Aurther Martin".to_string(),
                    color: Color32::from_rgb(220, 180, 80),
                    portrait_path: "portraits/king.png".to_string(),
                },
                CharacterEntity {
                    id: "mage".to_string(),
                    name: "Mage".to_string(),
                    color: Color32::from_rgb(140, 120, 240),
                    portrait_path: "portraits/mage.png".to_string(),
                },
                CharacterEntity {
                    id: "narrator".to_string(),
                    name: "Narrator".to_string(),
                    color: Color32::from_rgb(180, 180, 190),
                    portrait_path: "".to_string(),
                },
            ],
            audio_assets: vec![
                AudioEntry {
                    key: "KingAurtherMartin001.wav".to_string(),
                    filename: "KingAurtherMartin001.wav".to_string(),
                    speaker: "King Aurther Martin".to_string(),
                    duration_sec: 4.8,
                },
                AudioEntry {
                    key: "Mage001.wav".to_string(),
                    filename: "Mage001.wav".to_string(),
                    speaker: "Mage".to_string(),
                    duration_sec: 3.2,
                },
            ],
            scene_outline: vec![
                SceneOutlineItem {
                    id: 0,
                    title: "The Main Gate Encounter".to_string(),
                    chapter: "Act 1: The Gathering Storm".to_string(),
                },
                SceneOutlineItem {
                    id: 1,
                    title: "The Sorcerer's Riddle".to_string(),
                    chapter: "Act 1: The Gathering Storm".to_string(),
                },
            ],
        }
    }
}
