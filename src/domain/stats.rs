use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeStats {
    pub total_listened_count: u64,
    pub total_shadowed_count: u64,
    pub total_listening_secs: f64,
    pub today_date: String,
    pub today_sentences_count: u64,
    pub today_repetitions_count: u64,
    pub today_listening_secs: f64,
}

impl Default for PracticeStats {
    fn default() -> Self {
        Self {
            total_listened_count: 0,
            total_shadowed_count: 0,
            total_listening_secs: 0.0,
            today_date: Local::now().format("%Y-%m-%d").to_string(),
            today_sentences_count: 0,
            today_repetitions_count: 0,
            today_listening_secs: 0.0,
        }
    }
}

impl PracticeStats {
    pub fn check_day_rollover(&mut self) {
        let today = Local::now().format("%Y-%m-%d").to_string();
        if self.today_date != today {
            self.today_date = today;
            self.today_sentences_count = 0;
            self.today_repetitions_count = 0;
            self.today_listening_secs = 0.0;
        }
    }

    pub fn record_playback(&mut self, duration_secs: f64, is_shadow: bool) {
        self.check_day_rollover();
        self.total_listened_count += 1;
        self.total_listening_secs += duration_secs;
        self.today_repetitions_count += 1;
        self.today_listening_secs += duration_secs;

        if is_shadow {
            self.total_shadowed_count += 1;
        }
    }

    pub fn record_sentence_completed(&mut self) {
        self.check_day_rollover();
        self.today_sentences_count += 1;
    }
}
