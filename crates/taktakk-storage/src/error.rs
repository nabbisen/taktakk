//! Storage-layer error type.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migration(String),

    #[error("serialisation error: {0}")]
    Serialisation(#[from] serde_json::Error),

    #[error("object not found: {hash}")]
    ObjectNotFound { hash: String },

    #[error("hash mismatch for object {hash}")]
    HashMismatch { hash: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type StorageResult<T> = Result<T, StorageError>;
