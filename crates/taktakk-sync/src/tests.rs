//! Unit and integration tests for taktakk-sync (M6).

use crate::chunk::{
    pending_chunk_indices, reassemble_chunks, split_into_chunks, verify_chunk,
    ChunkStatus, DEFAULT_CHUNK_SIZE,
};
use crate::import::{scan_directory, source_label_hash, ImportStatus, ItemVerifyResult};
use crate::inventory::LocalInventory;
use crate::manifest::{build_transfer_plan, deserialise_inventory, serialise_inventory, SyncAction};
use crate::permission::{build_requests, required_permissions, AppPermission, TriggerAction};
use crate::transport::local::{stage_objects, LocalFsTransport};
use crate::transport::SyncTransportAdapter;

// ── Helpers ───────────────────────────────────────────────────────────────────

struct TempDir(std::path::PathBuf);
impl TempDir {
    fn new(tag: &str) -> Self {
        let p = std::env::temp_dir()
            .join(format!("taktakk-sync-{tag}-{}", rand_ns()));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn path(&self) -> &std::path::Path { &self.0 }
}
impl Drop for TempDir {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
}
fn rand_ns() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as u64
}

fn inv(pkgs: &[(&str, &str, &str)]) -> LocalInventory {
    LocalInventory::build(
        pkgs.iter()
            .map(|(id, ver, hash)| (id.to_string(), ver.to_string(), hash.to_string()))
            .collect(),
    )
}

// ── LocalInventory ────────────────────────────────────────────────────────────

#[test]
fn inventory_missing_from_remote() {
    let local  = inv(&[("pkg-a", "1.0.0", "aaa"), ("pkg-b", "1.0.0", "bbb")]);
    let remote = inv(&[("pkg-a", "1.0.0", "aaa"), ("pkg-c", "1.0.0", "ccc")]);

    let missing = local.missing_from_remote(&remote);
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].package_id, "pkg-c");
}

#[test]
fn inventory_in_sync_when_equal() {
    let a = inv(&[("pkg-a", "1.0.0", "aaa")]);
    let b = inv(&[("pkg-a", "1.0.0", "aaa")]);
    assert!(a.is_in_sync_with(&b));
}

#[test]
fn inventory_not_in_sync_when_different() {
    let a = inv(&[("pkg-a", "1.0.0", "aaa")]);
    let b = inv(&[("pkg-b", "1.0.0", "bbb")]);
    assert!(!a.is_in_sync_with(&b));
}

#[test]
fn inventory_empty_builds_ok() {
    let i = inv(&[]);
    assert!(i.items.is_empty());
}

#[test]
fn inventory_round_trips_json() {
    let original = inv(&[("pkg-a", "1.0.0", "abc"), ("pkg-b", "2.0.0", "def")]);
    let json = serialise_inventory(&original).unwrap();
    let restored = deserialise_inventory(&json).unwrap();
    assert_eq!(original.snapshot_hash, restored.snapshot_hash);
    assert_eq!(restored.items.len(), 2);
}

// ── Manifest / transfer plan ──────────────────────────────────────────────────

#[test]
fn plan_receive_when_only_remote_has_package() {
    let local  = inv(&[("pkg-a", "1.0.0", "aaa")]);
    let remote = inv(&[("pkg-a", "1.0.0", "aaa"), ("pkg-b", "1.0.0", "bbb")]);
    let plan = build_transfer_plan(&local, &remote);

    let receive: Vec<_> = plan.iter().filter(|i| i.action == SyncAction::Receive).collect();
    assert_eq!(receive.len(), 1);
    assert_eq!(receive[0].package_id, "pkg-b");
}

#[test]
fn plan_send_when_only_local_has_package() {
    let local  = inv(&[("pkg-a", "1.0.0", "aaa"), ("pkg-b", "1.0.0", "bbb")]);
    let remote = inv(&[("pkg-a", "1.0.0", "aaa")]);
    let plan = build_transfer_plan(&local, &remote);

    let send: Vec<_> = plan.iter().filter(|i| i.action == SyncAction::Send).collect();
    assert_eq!(send.len(), 1);
    assert_eq!(send[0].package_id, "pkg-b");
}

#[test]
fn plan_skip_when_both_have_same_hash() {
    let a = inv(&[("pkg-a", "1.0.0", "aaa")]);
    let b = inv(&[("pkg-a", "1.0.0", "aaa")]);
    let plan = build_transfer_plan(&a, &b);
    assert!(plan.iter().all(|i| i.action == SyncAction::Skip));
}

#[test]
fn plan_verify_only_when_hash_differs() {
    let local  = inv(&[("pkg-a", "1.0.0", "aaa")]);
    let remote = inv(&[("pkg-a", "1.0.0", "bbb")]); // different hash
    let plan = build_transfer_plan(&local, &remote);
    let verify: Vec<_> = plan.iter().filter(|i| i.action == SyncAction::VerifyOnly).collect();
    assert_eq!(verify.len(), 1);
}

#[test]
fn plan_receive_items_come_first() {
    let local  = inv(&[("pkg-a", "1.0.0", "aaa")]);
    let remote = inv(&[("pkg-a", "1.0.0", "aaa"), ("pkg-b", "1.0.0", "bbb")]);
    let plan = build_transfer_plan(&local, &remote);
    // First non-Skip item should be Receive.
    let first_action = plan.first().map(|i| &i.action).unwrap();
    assert_eq!(*first_action, SyncAction::Receive);
}

// ── Chunk transfer ────────────────────────────────────────────────────────────

#[test]
fn split_and_reassemble_small_object() {
    let data = b"hello taktakk sync chunk model";
    use sha2::{Digest, Sha256};
    let obj_hash = hex::encode(Sha256::digest(data));

    let chunks = split_into_chunks("tx-1", &obj_hash, data, DEFAULT_CHUNK_SIZE, 0);
    assert_eq!(chunks.len(), 1);

    let chunk_data: Vec<(u32, Vec<u8>)> = vec![(0, data.to_vec())];
    let reassembled = reassemble_chunks(&chunk_data, &chunks).unwrap();
    assert_eq!(reassembled, data);
}

#[test]
fn split_multi_chunk_object() {
    let data: Vec<u8> = (0u8..=255u8).cycle().take(3 * 1024).collect();
    use sha2::{Digest, Sha256};
    let obj_hash = hex::encode(Sha256::digest(&data));

    let chunks = split_into_chunks("tx-2", &obj_hash, &data, 1024, 0);
    assert_eq!(chunks.len(), 3);
}

#[test]
fn verify_chunk_correct_hash() {
    let data = b"chunk data";
    let hash = hex::encode(sha2::Digest::finalize(sha2::Sha256::new_with_prefix(data)));
    // Use the real sha2 API
    use sha2::{Digest, Sha256};
    let real_hash = hex::encode(Sha256::digest(data));
    assert!(verify_chunk(data, &real_hash));
}

#[test]
fn verify_chunk_wrong_hash_fails() {
    assert!(!verify_chunk(b"data", "deadbeef"));
}

#[test]
fn reassemble_tampered_chunk_fails() {
    let data = b"original data";
    use sha2::{Digest, Sha256};
    let obj_hash = hex::encode(Sha256::digest(data));
    let chunks = split_into_chunks("tx-3", &obj_hash, data, 64, 0);

    // Tamper with the chunk data.
    let tampered: Vec<(u32, Vec<u8>)> = vec![(0, b"tampered data".to_vec())];
    assert!(reassemble_chunks(&tampered, &chunks).is_err());
}

#[test]
fn pending_chunks_returned_correctly() {
    let data = (0u8..128).collect::<Vec<_>>();
    use sha2::{Digest, Sha256};
    let hash = hex::encode(Sha256::digest(&data));
    let mut chunks = split_into_chunks("tx-4", &hash, &data, 64, 0);

    // Mark first chunk as Verified.
    chunks[0].status = ChunkStatus::Verified;

    let pending = pending_chunk_indices(&chunks);
    assert_eq!(pending, vec![1]);
}

// ── Physical media import ─────────────────────────────────────────────────────

#[test]
fn scan_directory_finds_nmp_files() {
    let dir = TempDir::new("scan");

    // Create a valid .nmp file (magic bytes TAKT).
    std::fs::write(dir.path().join("test.nmp"), b"TAKT\x01\x00\x00\x00\x00").unwrap();
    // Create a non-.nmp file (should be ignored).
    std::fs::write(dir.path().join("other.txt"), b"hello").unwrap();

    let result = scan_directory(dir.path(), 3);
    assert_eq!(result.found.len(), 1);
    assert!(result.found[0].looks_valid);
}

#[test]
fn scan_directory_detects_invalid_magic() {
    let dir = TempDir::new("scan-bad");
    std::fs::write(dir.path().join("bad.nmp"), b"JPEG\xff\xd8").unwrap();
    let result = scan_directory(dir.path(), 2);
    assert_eq!(result.found.len(), 1);
    assert!(!result.found[0].looks_valid);
}

#[test]
fn scan_directory_recursive() {
    let dir = TempDir::new("scan-rec");
    let sub = dir.path().join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(sub.join("deep.nmp"), b"TAKT\x01\x00\x00\x00\x00").unwrap();
    let result = scan_directory(dir.path(), 5);
    assert_eq!(result.found.len(), 1);
}

#[test]
fn scan_empty_directory_returns_empty() {
    let dir = TempDir::new("scan-empty");
    let result = scan_directory(dir.path(), 3);
    assert!(result.found.is_empty());
    assert_eq!(result.scan_error_count, 0);
}

#[test]
fn source_label_hash_does_not_expose_path() {
    let path = std::path::Path::new("/storage/sdcard0/secret/taktakk.nmp");
    let h = source_label_hash(path);
    assert!(!h.contains("storage"));
    assert!(!h.contains("secret"));
    assert_eq!(h.len(), 64); // SHA-256 hex
}

#[test]
fn source_label_hash_is_deterministic() {
    let path = std::path::Path::new("/mnt/usb/pkg.nmp");
    assert_eq!(source_label_hash(path), source_label_hash(path));
}

// ── Permission timing ─────────────────────────────────────────────────────────

#[test]
fn share_menu_requires_bluetooth_and_network() {
    let perms = required_permissions(&TriggerAction::OpenShareMenu);
    assert!(perms.contains(&AppPermission::BluetoothNearbyDevices));
    assert!(perms.contains(&AppPermission::LocalNetwork));
}

#[test]
fn import_requires_storage() {
    let perms = required_permissions(&TriggerAction::OpenImportFromStorage);
    assert_eq!(perms, vec![AppPermission::ExternalStorageRead]);
}

#[test]
fn audio_requires_only_audio_permission() {
    let perms = required_permissions(&TriggerAction::PlayAudio);
    assert_eq!(perms, vec![AppPermission::AudioPlayback]);
}

#[test]
fn permission_requests_are_unlocked_shell_only() {
    let requests = build_requests(&TriggerAction::OpenShareMenu);
    assert!(requests.iter().all(|r| r.unlocked_only));
}

#[test]
fn explanation_keys_do_not_contain_taktakk() {
    use crate::permission::user_safe_explanation_key;
    for perm in &[
        AppPermission::BluetoothNearbyDevices,
        AppPermission::LocalNetwork,
        AppPermission::ExternalStorageRead,
        AppPermission::AudioPlayback,
        AppPermission::Camera,
    ] {
        let key = user_safe_explanation_key(perm);
        assert!(!key.to_lowercase().contains("taktakk"),
            "explanation key should not mention taktakk: {key}");
        assert!(!key.to_lowercase().contains("learn"),
            "explanation key should not mention learning: {key}");
    }
}

// ── Local transport: full end-to-end ─────────────────────────────────────────

#[test]
fn local_transport_push_and_fetch_object() {
    let dir = TempDir::new("transport");
    let ta = LocalFsTransport::new(dir.path().to_path_buf(), "a");

    let data = b"taktakk transport test object";
    use sha2::{Digest, Sha256};
    let hash = hex::encode(Sha256::digest(data));

    ta.push_object(&hash, data).unwrap();
    let fetched = ta.fetch_object(&hash).unwrap();
    assert_eq!(fetched, data);
}

#[test]
fn local_transport_inventory_exchange() {
    let dir = TempDir::new("exchange");
    let ta = LocalFsTransport::new(dir.path().to_path_buf(), "a");
    let tb = LocalFsTransport::new(dir.path().to_path_buf(), "b");

    let inv_a = inv(&[("pkg-a", "1.0.0", "aaa")]);
    let inv_b = inv(&[("pkg-b", "1.0.0", "bbb")]);

    let json_a = serialise_inventory(&inv_a).unwrap();
    let json_b = serialise_inventory(&inv_b).unwrap();

    // A writes, B writes; now each can read the other.
    ta.exchange_inventory(&json_a).unwrap_err(); // peer not yet written
    tb.exchange_inventory(&json_b).unwrap();     // B writes; A's file exists too now? No...

    // Write both sides manually so the exchange works.
    std::fs::write(dir.path().join("a_inventory.json"), &json_a).unwrap();
    std::fs::write(dir.path().join("b_inventory.json"), &json_b).unwrap();

    let received_by_a = ta.exchange_inventory(&json_a).unwrap();
    let received_by_b = tb.exchange_inventory(&json_b).unwrap();

    let parsed_a = deserialise_inventory(&received_by_a).unwrap();
    let parsed_b = deserialise_inventory(&received_by_b).unwrap();

    assert_eq!(parsed_a.items[0].package_id, "pkg-b");
    assert_eq!(parsed_b.items[0].package_id, "pkg-a");
}

#[test]
fn local_transport_stage_objects() {
    let dir = TempDir::new("stage");
    use sha2::{Digest, Sha256};

    let data1 = b"object one";
    let data2 = b"object two";
    let h1 = hex::encode(Sha256::digest(data1));
    let h2 = hex::encode(Sha256::digest(data2));

    stage_objects(dir.path(), &[(&h1, data1), (&h2, data2)]).unwrap();

    let ta = LocalFsTransport::new(dir.path().to_path_buf(), "a");
    assert_eq!(ta.fetch_object(&h1).unwrap(), data1);
    assert_eq!(ta.fetch_object(&h2).unwrap(), data2);
}
