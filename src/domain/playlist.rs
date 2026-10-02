use crate::domain::sentence::{CardStatus, Sentence};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaylistFilter {
    AllReviewed,
    Mature,
    Young,
    RecentlyReviewed,
    Deck(String),
    Tag(String),
    Search(String),
}

impl PlaylistFilter {
    pub fn name(&self) -> String {
        match self {
            Self::AllReviewed => "All Reviewed".to_string(),
            Self::Mature => "Mature (ivl >= 21d)".to_string(),
            Self::Young => "Young (ivl < 21d)".to_string(),
            Self::RecentlyReviewed => "Recently Reviewed".to_string(),
            Self::Deck(d) => format!("Deck: {d}"),
            Self::Tag(t) => format!("Tag: {t}"),
            Self::Search(q) => format!("Search: {q}"),
        }
    }

    pub fn matches(&self, sentence: &Sentence) -> bool {
        match self {
            Self::AllReviewed => {
                sentence.status == CardStatus::Mature || sentence.status == CardStatus::Young
            }
            Self::Mature => sentence.status == CardStatus::Mature,
            Self::Young => sentence.status == CardStatus::Young,
            Self::RecentlyReviewed => sentence.last_reviewed.is_some(),
            Self::Deck(d) => sentence.deck.eq_ignore_ascii_case(d),
            Self::Tag(t) => sentence.tags.iter().any(|tag| tag.eq_ignore_ascii_case(t)),
            Self::Search(q) => {
                let query = q.to_lowercase();
                sentence.japanese_text.to_lowercase().contains(&query)
                    || sentence
                        .translation
                        .as_ref()
                        .is_some_and(|tr| tr.to_lowercase().contains(&query))
                    || sentence
                        .furigana
                        .as_ref()
                        .is_some_and(|fg| fg.to_lowercase().contains(&query))
                    || sentence
                        .tags
                        .iter()
                        .any(|tg| tg.to_lowercase().contains(&query))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub filter: PlaylistFilter,
}

impl Playlist {
    pub fn new(name: impl Into<String>, filter: PlaylistFilter) -> Self {
        Self {
            name: name.into(),
            filter,
        }
    }

    pub fn apply(&self, pool: &[Sentence]) -> Vec<Sentence> {
        let mut filtered: Vec<Sentence> = pool
            .iter()
            .filter(|s| self.filter.matches(s))
            .cloned()
            .collect();

        if let PlaylistFilter::RecentlyReviewed = self.filter {
            filtered.sort_by_key(|b| std::cmp::Reverse(b.last_reviewed));
        }

        filtered
    }
}
