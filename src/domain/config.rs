use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_shadow_breath() -> f32 {
    1.2
}

fn default_echo_breath() -> f32 {
    1.0
}

fn default_card_complete_break() -> f32 {
    2.0
}

fn default_listen_gap() -> f32 {
    1.2
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackPreset {
    Shadow,
    #[serde(alias = "Repeat")]
    Echo,
    Listen,
}

impl PlaybackPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Shadow => "Shadow",
            Self::Echo => "Echo",
            Self::Listen => "Listen",
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            Self::Shadow => Self::Echo,
            Self::Echo => Self::Listen,
            Self::Listen => Self::Shadow,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub preset: PlaybackPreset,
    pub repeat_count: u32,
    pub pause_after_replay_secs: f32,
    pub auto_advance: bool,
    pub show_japanese: bool,
    pub show_furigana: bool,
    pub show_translation: bool,
    pub volume: f32,
    pub anki_collection_path: Option<PathBuf>,
    pub anki_media_path: Option<PathBuf>,
    #[serde(default = "default_shadow_breath")]
    pub shadow_breath_gap_secs: f32,
    #[serde(default = "default_echo_breath")]
    pub echo_breath_gap_secs: f32,
    #[serde(default = "default_card_complete_break")]
    pub card_complete_break_secs: f32,
    #[serde(default = "default_listen_gap")]
    pub listen_gap_secs: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            preset: PlaybackPreset::Shadow,
            repeat_count: 2,
            pause_after_replay_secs: 1.0,
            auto_advance: true,
            show_japanese: false,
            show_furigana: false,
            show_translation: false,
            volume: 1.0,
            anki_collection_path: None,
            anki_media_path: None,
            shadow_breath_gap_secs: 1.2,
            echo_breath_gap_secs: 1.0,
            card_complete_break_secs: 2.0,
            listen_gap_secs: 1.2,
        }
    }
}
