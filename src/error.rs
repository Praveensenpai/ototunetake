use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Audio playback error: {0}")]
    Audio(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Anki collection not found: {0}")]
    CollectionNotFound(PathBuf),

    #[error("Anki media directory not found: {0}")]
    MediaNotFound(PathBuf),

    #[error("Queue is empty")]
    EmptyQueue,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
