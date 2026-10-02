use crate::domain::config::AppConfig;
use crate::domain::stats::PracticeStats;
use crate::error::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub last_sentence_id: Option<i64>,
    pub playlist_name: String,
    pub volume: f32,
    pub shuffle: bool,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            last_sentence_id: None,
            playlist_name: "All Reviewed".to_string(),
            volume: 1.0,
            shuffle: false,
        }
    }
}

pub struct AppStorage {
    config_dir: PathBuf,
}

impl AppStorage {
    pub fn new() -> Result<Self> {
        let proj_dirs =
            ProjectDirs::from("com", "praveensenpai", "ototunetake").ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "No home directory")
            })?;
        let config_dir = proj_dirs.config_dir().to_path_buf();

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        Ok(Self { config_dir })
    }

    pub fn load_config(&self) -> AppConfig {
        let path = self.config_dir.join("config.json");
        if let Ok(file) = File::open(&path) {
            let reader = BufReader::new(file);
            if let Ok(cfg) = serde_json::from_reader(reader) {
                return cfg;
            }
        }
        AppConfig::default()
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<()> {
        let path = self.config_dir.join("config.json");
        let json = serde_json::to_string_pretty(config)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_stats(&self) -> PracticeStats {
        let path = self.config_dir.join("stats.json");
        if let Ok(file) = File::open(&path) {
            let reader = BufReader::new(file);
            if let Ok(mut stats) = serde_json::from_reader::<_, PracticeStats>(reader) {
                stats.check_day_rollover();
                return stats;
            }
        }
        PracticeStats::default()
    }

    pub fn save_stats(&self, stats: &PracticeStats) -> Result<()> {
        let path = self.config_dir.join("stats.json");
        let json = serde_json::to_string_pretty(stats)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_session(&self) -> SessionState {
        let path = self.config_dir.join("session.json");
        if let Ok(file) = File::open(&path) {
            let reader = BufReader::new(file);
            if let Ok(session) = serde_json::from_reader(reader) {
                return session;
            }
        }
        SessionState::default()
    }

    pub fn save_session(&self, session: &SessionState) -> Result<()> {
        let path = self.config_dir.join("session.json");
        let json = serde_json::to_string_pretty(session)?;
        fs::write(path, json)?;
        Ok(())
    }
}
