use crate::domain::playlist::PlaylistFilter;
use crate::domain::sentence::Sentence;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct SentenceQueue {
    pool: Vec<Sentence>,
    active_sentences: Vec<Sentence>,
    cursor: usize,
    shuffle: bool,
    active_filter: PlaylistFilter,
}

impl SentenceQueue {
    pub fn new(pool: Vec<Sentence>) -> Self {
        let filter = PlaylistFilter::AllReviewed;
        let mut active = Vec::new();
        for item in &pool {
            if filter.matches(item) {
                active.push(item.clone());
            }
        }

        Self {
            pool,
            active_sentences: active,
            cursor: 0,
            shuffle: false,
            active_filter: filter,
        }
    }

    pub fn current(&self) -> Option<&Sentence> {
        self.active_sentences.get(self.cursor)
    }

    pub fn current_mut(&mut self) -> Option<&mut Sentence> {
        self.active_sentences.get_mut(self.cursor)
    }

    pub fn advance(&mut self) -> Option<&Sentence> {
        if self.active_sentences.is_empty() {
            return None;
        }
        if self.cursor + 1 < self.active_sentences.len() {
            self.cursor += 1;
        } else {
            self.cursor = 0;
        }
        self.current()
    }

    pub fn previous(&mut self) -> Option<&Sentence> {
        if self.active_sentences.is_empty() {
            return None;
        }
        if self.cursor > 0 {
            self.cursor -= 1;
        } else {
            self.cursor = self.active_sentences.len().saturating_sub(1);
        }
        self.current()
    }

    pub fn jump_to(&mut self, index: usize) -> Option<&Sentence> {
        if index < self.active_sentences.len() {
            self.cursor = index;
            self.current()
        } else {
            None
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        let current_id = self.current().map(|s| s.id);

        if self.shuffle {
            let mut rng = thread_rng();
            self.active_sentences.shuffle(&mut rng);
        } else {
            self.reapply_filter();
        }

        if let Some(id) = current_id {
            if let Some(pos) = self.active_sentences.iter().position(|s| s.id == id) {
                self.cursor = pos;
            }
        }
    }

    pub fn is_shuffled(&self) -> bool {
        self.shuffle
    }

    pub fn set_filter(&mut self, filter: PlaylistFilter) {
        self.active_filter = filter;
        self.reapply_filter();
        self.cursor = 0;
    }

    pub fn active_filter(&self) -> &PlaylistFilter {
        &self.active_filter
    }

    pub fn reapply_filter(&mut self) {
        let mut filtered: Vec<Sentence> = self
            .pool
            .iter()
            .filter(|s| self.active_filter.matches(s))
            .cloned()
            .collect();

        if let PlaylistFilter::RecentlyReviewed = self.active_filter {
            filtered.sort_by_key(|b| std::cmp::Reverse(b.last_reviewed));
        }

        self.active_sentences = filtered;
    }

    pub fn upcoming(&self, limit: usize) -> Vec<&Sentence> {
        if self.active_sentences.is_empty() {
            return Vec::new();
        }
        let total = self.active_sentences.len();
        (1..=limit)
            .map(|offset| &self.active_sentences[(self.cursor + offset) % total])
            .collect()
    }

    pub fn remove_at(&mut self, index: usize) -> Option<Sentence> {
        if index >= self.active_sentences.len() {
            return None;
        }
        let removed = self.active_sentences.remove(index);
        if self.cursor >= self.active_sentences.len() && !self.active_sentences.is_empty() {
            self.cursor = self.active_sentences.len() - 1;
        }
        Some(removed)
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn total(&self) -> usize {
        self.active_sentences.len()
    }

    pub fn is_empty(&self) -> bool {
        self.active_sentences.is_empty()
    }

    pub fn pool(&self) -> &[Sentence] {
        &self.pool
    }

    pub fn active_sentences(&self) -> &[Sentence] {
        &self.active_sentences
    }
}
