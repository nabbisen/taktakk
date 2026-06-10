//! Content-layer error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContentError {
    #[error("invalid magic bytes — not a .nmp package")]
    BadMagic,

    // Alias for BadMagic used by stream_reader
    #[error("invalid magic bytes — not a .nmp package")]
    InvalidMagic,

    #[error("unsupported format version: {0}")]
    UnsupportedVersion(u8),

    #[error("manifest too large: {size} bytes (max {max})")]
    ManifestTooLarge { size: u32, max: u32 },

    #[error("manifest JSON parse error: {0}")]
    ManifestParse(String),

    #[error("package parse error: {0}")]
    Parse(String),

    #[error("signature verification failed: {0}")]
    SignatureVerification(String),

    #[error("signature verification failed")]
    SignatureFailed,

    #[error("signer '{0}' not found in trust anchors")]
    UnknownSigner(String),

    #[error("object hash mismatch for '{path}': expected {expected}, got {actual}")]
    HashMismatch { path: String, expected: String, actual: String },

    #[error("object count mismatch: manifest has {manifest}, package has {actual}")]
    ObjectCountMismatch { manifest: usize, actual: usize },

    #[error("package truncated at offset {offset}")]
    Truncated { offset: usize },

    #[error("incompatible package: requires core {required}, have {current}")]
    IncompatibleVersion { required: String, current: String },

    #[error("I/O error: {0}")]
    Io(String),
}

impl From<std::io::Error> for ContentError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

pub type ContentResult<T> = Result<T, ContentError>;
