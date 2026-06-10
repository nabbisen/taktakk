//! Package manifest utilities.

use taktakk_core::domain::package::PackageManifest;

/// Validate a parsed manifest for structural correctness.
pub fn validate_manifest(m: &PackageManifest) -> Result<(), String> {
    if m.module_id.is_empty() {
        return Err("module_id is empty".to_string());
    }
    if m.signer_id.is_empty() {
        return Err("signer_id is empty".to_string());
    }
    for obj in &m.objects {
        if obj.sha256.len() != 64 {
            return Err(format!(
                "object '{}' has invalid sha256 length {}",
                obj.path,
                obj.sha256.len()
            ));
        }
        if !obj.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("object '{}' sha256 contains non-hex chars", obj.path));
        }
    }
    Ok(())
}
