//! Panic wipe coordinator implementation (RFC 018).

use rand::RngCore;
use taktakk_core::error::CoreResult;
use taktakk_core::ports::crypto::WipeCoordinator;

use crate::key_slot::{CryptoKeySlot, KeyStatus};

/// Overwrite a key slot with 7 passes of random noise and mark destroyed.
pub fn overwrite_key_slot(slot: &mut CryptoKeySlot) {
    let mut rng = rand::thread_rng();
    for _ in 0..7 {
        rng.fill_bytes(&mut slot.wrapped_key);
    }
    slot.status = KeyStatus::Destroyed;
}

/// Overwrite all active or retired key slots. Returns count destroyed.
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

/// Log-redaction: replace printable characters with asterisks, preserve newlines.
pub fn redact(input: &str) -> String {
    input.chars().map(|c| if c == '\n' { c } else { '*' }).collect()
}

/// Return `true` if the log tag is safe to persist (no domain words).
pub fn is_safe_log_tag(tag: &str) -> bool {
    if tag.len() > 20 { return false; }
    let banned = ["shield", "spear", "module", "lesson", "profile",
                  "package", "install", "sync", "user", "learn"];
    let lower = tag.to_lowercase();
    !banned.iter().any(|b| lower.contains(b))
}
