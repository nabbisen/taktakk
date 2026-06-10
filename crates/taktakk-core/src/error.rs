//! Core error types.

use thiserror::Error;

/// Top-level error type for taktakk-core operations.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("package not found: {id}")]
    PackageNotFound { id: String },

    #[error("signature verification failed")]
    SignatureVerificationFailed,

    #[error("content hash mismatch for object {hash}")]
    HashMismatch { hash: String },

    #[error("unsupported package format version: {version}")]
    UnsupportedVersion { version: u8 },

    #[error("unlock sequence rejected")]
    UnlockRejected,

    #[error("wipe already in progress or completed")]
    WipeConflict,

    #[error("module not found: {id}")]
    ModuleNotFound { id: String },

    #[error("lesson not found: {id}")]
    LessonNotFound { id: String },

    #[error("profile not initialized")]
    ProfileNotInitialized,

    #[error("storage error: {0}")]
    Storage(String),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("sync error: {0}")]
    Sync(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
