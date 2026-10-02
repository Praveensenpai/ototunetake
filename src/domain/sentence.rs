use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardStatus {
    Mature,
    Young,
    New,
    Suspended,
}

impl CardStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mature => "Mature",
            Self::Young => "Young",
            Self::New => "New",
            Self::Suspended => "Suspended",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentence {
    pub id: i64,
    pub card_id: i64,
    pub note_id: i64,
    pub japanese_text: String,
    pub furigana: Option<String>,
    pub translation: Option<String>,
    pub audio_path: PathBuf,
    pub deck: String,
    pub tags: Vec<String>,
    pub status: CardStatus,
    pub interval: i64,
    pub reps: i64,
    pub last_reviewed: Option<i64>,
    pub play_count: u64,
    pub shadow_count: u64,
    pub last_played: Option<i64>,
}

impl Sentence {
    pub fn display_tags(&self) -> String {
        if self.tags.is_empty() {
            "none".to_string()
        } else {
            self.tags.join(", ")
        }
    }
}
