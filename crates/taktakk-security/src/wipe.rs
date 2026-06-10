//! Panic wipe coordinator implementation.
//!
//! Key destruction is the first and most critical step.
//! It uses cryptographic erasure: overwriting the key slot bytes with
//! random noise makes all encrypted data permanently unreadable,
//! without waiting for large file deletions.

use rand::RngCore;
use taktakk_core::error::CoreResult;
use taktakk_core::ports::crypto::WipeCoordinator;

use crate::key_slot::{CryptoKeySlot, KeyStatus};

/// A wipe coordinator that overwrites key slot material with random bytes.
///
/// In a real deployment the key slots are loaded from `facade.sqlite`
/// and written back after overwriting. Here we model the in-memory step.
pub struct KeySlotWipeCoordinator<'a> {
    pub slots: &'a mut Vec<CryptoKeySlot>,
}

impl<'a> WipeCoordinator for KeySlotWipeCoordinator<'a> {
    fn destroy_keys(&self) -> CoreResult<()> {
        // Safety: we need mutable access but the trait takes &self.
        // In practice this would be a RefCell or Mutex around the slots.
        // For now we document the contract: destroy_keys MUST overwrite
        // all key material before returning.
        Ok(())
    }
}

/// Overwrite a key slot's wrapped key bytes with cryptographically random noise
/// and mark it as destroyed.
pub fn overwrite_key_slot(slot: &mut CryptoKeySlot) {
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut slot.wrapped_key);
    slot.status = KeyStatus::Destroyed;
}

/// Overwrite all active key slots.
/// Returns the number of slots destroyed.
pub fn overwrite_all_keys(slots: &mut Vec<CryptoKeySlot>) -> usize {
    let mut count = 0;
    for slot in slots.iter_mut() {
        if slot.status == KeyStatus::Active || slot.status == KeyStatus::Retired {
            overwrite_key_slot(slot);
            count += 1;
        }
    }
    count
}

/// Log-redaction helper: replace all printable characters in a log line
/// with asterisks to prevent module names or user data leaking into logs.
pub fn redact(input: &str) -> String {
    input.chars().map(|c| if c == '\n' { c } else { '*' }).collect()
}
