//! Unit tests for taktakk-storage.

use taktakk_core::ports::package_store::ObjectStore;
use crate::object_store::FsObjectStore;

fn temp_store() -> (FsObjectStore, tempdir_guard::TempDir) {
    let dir = tempdir_guard::TempDir::new();
    let store = FsObjectStore::new(dir.path().join("objects"));
    (store, dir)
}

/// Minimal temp-dir helper to avoid a crate dependency in tests.
mod tempdir_guard {
    use std::path::{Path, PathBuf};
    pub struct TempDir(PathBuf);
    impl TempDir {
        pub fn new() -> Self {
            let p = std::env::temp_dir()
                .join(format!("taktakk-test-{}", std::process::id()));
            std::fs::create_dir_all(&p).unwrap();
            Self(p)
        }
        pub fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[test]
fn put_and_get_round_trip() {
    let (store, _dir) = temp_store();
    let data = b"taktakk object store test";
    let hash = store.put(data).expect("put should succeed");
    assert_eq!(hash.len(), 64, "SHA-256 hex should be 64 chars");
    let retrieved = store.get(&hash).expect("get should succeed");
    assert_eq!(retrieved, data);
}

#[test]
fn exists_true_after_put() {
    let (store, _dir) = temp_store();
    let hash = store.put(b"hello").unwrap();
    assert!(store.exists(&hash).unwrap());
}

#[test]
fn exists_false_for_unknown_hash() {
    let (store, _dir) = temp_store();
    assert!(!store.exists("deadbeef".repeat(8).as_str()).unwrap());
}

#[test]
fn get_nonexistent_returns_err() {
    let (store, _dir) = temp_store();
    assert!(store.get("a".repeat(64).as_str()).is_err());
}

#[test]
fn delete_removes_object() {
    let (store, _dir) = temp_store();
    let hash = store.put(b"delete me").unwrap();
    store.delete(&hash).unwrap();
    assert!(!store.exists(&hash).unwrap());
}

#[test]
fn quarantine_moves_object() {
    let (store, _dir) = temp_store();
    let hash = store.put(b"bad data").unwrap();
    store.quarantine(&hash, "hash mismatch").unwrap();
    // Original path should no longer exist.
    assert!(!store.exists(&hash).unwrap());
}

#[test]
fn put_is_deterministic() {
    let (store, _dir) = temp_store();
    let h1 = store.put(b"same data").unwrap();
    let h2 = store.put(b"same data").unwrap();
    assert_eq!(h1, h2);
}
