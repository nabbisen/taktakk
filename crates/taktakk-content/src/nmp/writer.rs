//! `.nmp` package writer: build signed packages for testing and tooling.

use sha2::{Digest, Sha256};

use taktakk_core::domain::package::{NMP_FORMAT_VERSION, NMP_MAGIC, ObjectEntry, ObjectType, PackageManifest};

use super::error::{ContentError, ContentResult};

/// Builder for creating a signed `.nmp` package.
pub struct NmpWriter {
    manifest: PackageManifest,
    objects: Vec<(String, Vec<u8>, ObjectType)>, // (path, data, type)
}

impl NmpWriter {
    pub fn new(manifest: PackageManifest) -> Self {
        Self { manifest, objects: Vec::new() }
    }

    /// Add a named object. The SHA-256 hash is computed automatically and
    /// written into `manifest.objects`. The `object_type` is preserved.
    pub fn add_object(&mut self, path: impl Into<String>, data: Vec<u8>, object_type: ObjectType) {
        let path = path.into();
        let sha256 = hex::encode(Sha256::digest(&data));
        self.manifest.objects.push(ObjectEntry {
            path: path.clone(),
            sha256,
            object_type: object_type.clone(),
            required: true,
        });
        self.objects.push((path, data, object_type));
    }

    /// Serialise the manifest to JSON and return the bytes.
    pub fn manifest_bytes(&self) -> ContentResult<Vec<u8>> {
        serde_json::to_vec(&self.manifest)
            .map_err(|e| ContentError::ManifestParse(e.to_string()))
    }

    /// Build the final `.nmp` byte buffer, signing the manifest with
    /// the provided `sign` closure.
    ///
    /// `sign(manifest_bytes) -> [u8; 64]`
    pub fn build<F>(mut self, sign: F) -> ContentResult<Vec<u8>>
    where
        F: Fn(&[u8]) -> [u8; 64],
    {
        // Re-compute all object hashes to ensure consistency,
        // preserving the object_type set by add_object().
        self.manifest.objects.clear();
        for (path, data, object_type) in &self.objects {
            let sha256 = hex::encode(Sha256::digest(data));
            self.manifest.objects.push(ObjectEntry {
                path: path.clone(),
                sha256,
                object_type: object_type.clone(),
                required: true,
            });
        }

        let manifest_bytes = self.manifest_bytes()?;
        let signature = sign(&manifest_bytes);

        let mut buf = Vec::new();

        // Magic + version
        buf.extend_from_slice(&NMP_MAGIC);
        buf.push(NMP_FORMAT_VERSION);

        // Manifest
        let mlen = manifest_bytes.len() as u32;
        buf.extend_from_slice(&mlen.to_be_bytes());
        buf.extend_from_slice(&manifest_bytes);

        // Signature
        buf.extend_from_slice(&64u32.to_be_bytes());
        buf.extend_from_slice(&signature);

        // Object count
        let obj_count = self.objects.len() as u32;
        buf.extend_from_slice(&obj_count.to_be_bytes());

        // Objects
        for (_, data, _) in &self.objects {
            let dlen = data.len() as u32;
            buf.extend_from_slice(&dlen.to_be_bytes());
            buf.extend_from_slice(data);
        }

        Ok(buf)
    }
}
