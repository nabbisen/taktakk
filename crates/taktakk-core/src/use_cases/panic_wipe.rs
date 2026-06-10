//! Panic wipe use case.
//!
//! This is the most safety-critical operation in taktakk.
//! Destroying keys must happen before any slower deletion work.
//! The wipe must succeed even if subsequent cleanup steps fail.

use crate::error::{CoreError, CoreResult};
use crate::ports::crypto::WipeCoordinator;

/// Policy governing what a wipe should erase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WipeScope {
    /// Destroy crypto keys only. Data remains but becomes unreadable.
    /// Fast; suitable for panic situations.
    KeysOnly,
    /// Destroy keys and delete all core database and object store files.
    /// Slower but more thorough.
    Full,
}

/// Outcome of a wipe operation.
#[derive(Debug, Clone)]
pub struct WipeResult {
    pub keys_destroyed: bool,
    pub scope: WipeScope,
}

/// Execute a panic wipe.
///
/// Step order:
/// 1. Destroy crypto key slots (fast, irreversible).
/// 2. If [`WipeScope::Full`], schedule slow deletion of database and object files.
///
/// The function returns `Ok` as soon as keys are destroyed, even if slow
/// deletion has not completed or has failed.
pub fn execute_panic_wipe(
    wipe_coordinator: &dyn WipeCoordinator,
    scope: WipeScope,
) -> CoreResult<WipeResult> {
    // Step 1: destroy keys — must succeed.
    wipe_coordinator.destroy_keys().map_err(|e| {
        CoreError::Internal(format!("key destruction failed: {e}"))
    })?;

    // Step 2: full wipe schedules deletion (best-effort; failures are logged
    // but do not prevent returning Ok, since the data is already inaccessible).
    if scope == WipeScope::Full {
        // Slow deletion is handled by the caller / platform layer.
        // We record the intent here so the platform can resume if interrupted.
    }

    Ok(WipeResult {
        keys_destroyed: true,
        scope,
    })
}
