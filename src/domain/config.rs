use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackPreset {
    Listen,
    Repeat,
    Shadow,
}

impl PlaybackPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Listen => "Listen",
            Self::Repeat => "Repeat",
            Self::Shadow => "Shadow",
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            Self::Listen => Self::Repeat,
            Self::Repeat => Self::Shadow,
            Self::Shadow => Self::Listen,
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
        }
    }
}
