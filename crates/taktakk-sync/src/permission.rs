//! Delayed permission request model (RFC 004 / RFC 021).
//!
//! taktakk never requests permissions on launch. Permissions are requested
//! only when the user explicitly opens a feature that needs them.
//! This avoids triggering OS permission audits or raising suspicion during
//! inspection of the "Clock" app.
//!
//! Each permission is mapped to the earliest safe moment to request it.

use serde::{Deserialize, Serialize};

/// A system permission that taktakk may need.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppPermission {
    /// Needed to share/receive packages via Bluetooth.
    BluetoothNearbyDevices,
    /// Needed to share via Wi-Fi Direct / local network.
    LocalNetwork,
    /// Needed to import from SD card / USB (Android Storage Access Framework).
    ExternalStorageRead,
    /// Needed to play audio lessons.
    AudioPlayback,
    /// Needed for QR code scanning (camera).
    Camera,
}

/// The UI action that triggers a permission request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerAction {
    /// User opened the "Share" (sync) menu.
    OpenShareMenu,
    /// User tapped "Import from storage".
    OpenImportFromStorage,
    /// User tapped the audio play button on a lesson step.
    PlayAudio,
    /// User tapped the QR scanner button.
    OpenQrScanner,
}

/// Returns the minimum list of permissions needed for a trigger action.
pub fn required_permissions(action: &TriggerAction) -> Vec<AppPermission> {
    match action {
        TriggerAction::OpenShareMenu => vec![
            AppPermission::BluetoothNearbyDevices,
            AppPermission::LocalNetwork,
        ],
        TriggerAction::OpenImportFromStorage => vec![
            AppPermission::ExternalStorageRead,
        ],
        TriggerAction::PlayAudio => vec![
            AppPermission::AudioPlayback,
        ],
        TriggerAction::OpenQrScanner => vec![
            AppPermission::Camera,
        ],
    }
}

/// A user-safe explanation string key for why a permission is needed.
///
/// The explanation must not mention "taktakk", "learning", or any sensitive term.
pub fn user_safe_explanation_key(permission: &AppPermission) -> &'static str {
    match permission {
        AppPermission::BluetoothNearbyDevices =>
            "perm.bluetooth.reason",   // "To sync time with nearby devices"
        AppPermission::LocalNetwork =>
            "perm.network.reason",     // "To connect to local community devices"
        AppPermission::ExternalStorageRead =>
            "perm.storage.reason",     // "To read files from your storage card"
        AppPermission::AudioPlayback =>
            "perm.audio.reason",       // "To play audio content"
        AppPermission::Camera =>
            "perm.camera.reason",      // "To scan QR codes"
    }
}

/// A pending permission request.
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub permission: AppPermission,
    /// The i18n key for the user-facing explanation.
    pub explanation_key: &'static str,
    /// Whether to show the request in the *unlocked* shell only.
    pub unlocked_only: bool,
}

/// Build the set of permission requests for a trigger action.
pub fn build_requests(action: &TriggerAction) -> Vec<PermissionRequest> {
    required_permissions(action)
        .into_iter()
        .map(|p| {
            let key = user_safe_explanation_key(&p);
            PermissionRequest {
                permission: p,
                explanation_key: key,
                unlocked_only: true, // always request inside the unlocked shell
            }
        })
        .collect()
}
