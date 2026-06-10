//! Chunk transfer model for low-bandwidth / unreliable connections.
//!
//! Large objects are split into fixed-size chunks. Each chunk is individually
//! hashed and verified. An interrupted transfer can resume from the last
//! verified chunk without re-sending already-received data.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Default chunk size: 64 KiB — fits comfortably in RAM on low-end devices
/// while keeping per-chunk overhead small.
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Status of an individual chunk in a transfer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkStatus {
    Pending,
    Received,
    Verified,
    Failed,
}

/// A record for one chunk of an in-progress transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferChunk {
    pub transfer_id: String,
    /// SHA-256 hash of the full object being transferred.
    pub object_hash: String,
    pub chunk_index: u32,
    /// SHA-256 hash of this chunk's raw bytes.
    pub chunk_hash: String,
    pub byte_size: u32,
    pub status: ChunkStatus,
    pub updated_at: i64,
}

/// Split `data` into chunks of `chunk_size` bytes and compute per-chunk metadata.
pub fn split_into_chunks(
    transfer_id: &str,
    object_hash: &str,
    data: &[u8],
    chunk_size: usize,
    now: i64,
) -> Vec<TransferChunk> {
    data.chunks(chunk_size)
        .enumerate()
        .map(|(i, chunk)| {
            let chunk_hash = hex::encode(Sha256::digest(chunk));
            TransferChunk {
                transfer_id: transfer_id.to_string(),
                object_hash: object_hash.to_string(),
                chunk_index: i as u32,
                chunk_hash,
                byte_size: chunk.len() as u32,
                status: ChunkStatus::Pending,
                updated_at: now,
            }
        })
        .collect()
}

/// Verify a received chunk against its expected hash.
pub fn verify_chunk(chunk_data: &[u8], expected_hash: &str) -> bool {
    hex::encode(Sha256::digest(chunk_data)) == expected_hash
}

/// Reassemble chunks in index order, verifying each hash.
///
/// Returns `Ok(data)` if all chunks verify, or `Err(chunk_index)` on mismatch.
pub fn reassemble_chunks(
    chunks_data: &[(u32, Vec<u8>)],
    chunk_records: &[TransferChunk],
) -> Result<Vec<u8>, u32> {
    let mut sorted: Vec<&(u32, Vec<u8>)> = chunks_data.iter().collect();
    sorted.sort_by_key(|(idx, _)| *idx);

    let mut result = Vec::new();
    for (idx, data) in &sorted {
        let record = chunk_records
            .iter()
            .find(|r| r.chunk_index == *idx)
            .ok_or(*idx)?;
        if !verify_chunk(data, &record.chunk_hash) {
            return Err(*idx);
        }
        result.extend_from_slice(data);
    }
    Ok(result)
}

/// Return the indices of chunks that still need to be received.
pub fn pending_chunk_indices(chunks: &[TransferChunk]) -> Vec<u32> {
    chunks
        .iter()
        .filter(|c| c.status == ChunkStatus::Pending || c.status == ChunkStatus::Failed)
        .map(|c| c.chunk_index)
        .collect()
}
