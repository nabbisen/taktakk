//! Shared integration test harness.

use std::path::{Path, PathBuf};

use taktakk_content::fixtures::test_trust_anchor;
use taktakk_security::trust_anchor::TrustAnchor;
use taktakk_storage::db::Database;

/// An ephemeral, isolated test environment.
///
/// Each test gets its own temporary directory and database instance.
/// The directory is removed when the harness is dropped.
pub struct TestHarness {
    pub db: Database,
    pub data_dir: TempDir,
    pub trust_anchors: Vec<TrustAnchor>,
}

impl TestHarness {
    /// Create a new harness with an empty database and the test trust anchor.
    pub async fn new(tag: &str) -> Self {
        let data_dir = TempDir::new(tag);
        let db = Database::open(data_dir.path())
            .await
            .expect("TestHarness: database open");
        Self {
            db,
            data_dir,
            trust_anchors: vec![test_trust_anchor()],
        }
    }

    pub fn object_store_dir(&self) -> PathBuf {
        self.data_dir.path().join("objects")
    }

    pub fn object_store(&self) -> taktakk_storage::object_store::FsObjectStore {
        taktakk_storage::object_store::FsObjectStore::new(self.object_store_dir())
    }
}

/// Minimal ephemeral directory — removed on drop.
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new(tag: &str) -> Self {
        let p = std::env::temp_dir()
            .join(format!("taktakk-int-{tag}-{}", ns()));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }

    pub fn path(&self) -> &Path { &self.0 }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn ns() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as u64
}
