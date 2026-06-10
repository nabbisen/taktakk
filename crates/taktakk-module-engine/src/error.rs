//! Engine error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("no lesson loaded")]
    NoLesson,
    #[error("already at first step")]
    AtFirstStep,
    #[error("lesson already completed")]
    AlreadyCompleted,
    #[error("invalid exercise answer")]
    InvalidAnswer,
    #[error("state serialization error: {0}")]
    Serialization(String),
    #[error("step index out of range: {index} (len {len})")]
    StepOutOfRange { index: usize, len: usize },
}

pub type EngineResult<T> = Result<T, EngineError>;
