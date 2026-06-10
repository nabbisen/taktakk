//! Sync session and import job repository.

use sqlx::SqlitePool;

use taktakk_core::domain::sync::{SyncSession, SyncStatus, TransportKind};
use taktakk_core::error::{CoreError, CoreResult};

// ── Sync sessions ─────────────────────────────────────────────────────────────

pub async fn save_sync_session(pool: &SqlitePool, s: &SyncSession) -> CoreResult<()> {
    sqlx::query(
        "INSERT INTO sync_sessions
             (sync_session_id, transport, peer_ephemeral_hash,
              started_at, ended_at, status, packages_received, packages_sent)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(sync_session_id) DO UPDATE SET
             ended_at          = excluded.ended_at,
             status            = excluded.status,
             packages_received = excluded.packages_received,
             packages_sent     = excluded.packages_sent",
    )
    .bind(&s.session_id)
    .bind(transport_str(&s.transport))
    .bind(&s.peer_ephemeral_id)
    .bind(s.started_at)
    .bind(s.completed_at)
    .bind(sync_status_str(&s.status))
    .bind(s.objects_received)
    .bind(s.objects_sent)
    .execute(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

pub async fn get_sync_session(
    pool: &SqlitePool,
    session_id: &str,
) -> CoreResult<Option<SyncSession>> {
    type Row = (String, String, String, i64, Option<i64>, String, i64, i64);
    let row = sqlx::query_as::<_, Row>(
        "SELECT sync_session_id, transport, peer_ephemeral_hash,
                started_at, ended_at, status, packages_received, packages_sent
         FROM sync_sessions WHERE sync_session_id = ?",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;

    Ok(row.map(|(session_id, transport, peer, started_at, ended_at, status, recv, sent)| {
        SyncSession {
            session_id,
            peer_ephemeral_id: peer,
            transport: parse_transport(&transport),
            started_at,
            completed_at: ended_at,
            status: parse_sync_status(&status),
            objects_received: recv as u32,
            objects_sent: sent as u32,
        }
    }))
}

/// Enforce sync session retention (delete sessions older than `retention_seconds`).
pub async fn purge_old_sessions(
    pool: &SqlitePool,
    now: i64,
    retention_seconds: i64,
) -> CoreResult<u64> {
    let cutoff = now - retention_seconds;
    let r = sqlx::query(
        "DELETE FROM sync_sessions WHERE ended_at IS NOT NULL AND ended_at < ?",
    )
    .bind(cutoff)
    .execute(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(r.rows_affected())
}

// ── Import jobs ───────────────────────────────────────────────────────────────

/// Record the start of an import job.
pub async fn start_import_job(
    pool: &SqlitePool,
    import_job_id: &str,
    source_kind: &str,
    source_label_hash: Option<&str>,
    now: i64,
) -> CoreResult<()> {
    sqlx::query(
        "INSERT INTO import_jobs
             (import_job_id, source_kind, source_label_hash, started_at, status)
         VALUES (?, ?, ?, ?, 'scanning')",
    )
    .bind(import_job_id)
    .bind(source_kind)
    .bind(source_label_hash)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

/// Update import job status and counts.
pub async fn complete_import_job(
    pool: &SqlitePool,
    import_job_id: &str,
    installed_count: u32,
    now: i64,
) -> CoreResult<()> {
    sqlx::query(
        "UPDATE import_jobs
         SET status = 'installed', completed_at = ?, installed_count = ?
         WHERE import_job_id = ?",
    )
    .bind(now)
    .bind(installed_count)
    .bind(import_job_id)
    .execute(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn transport_str(t: &TransportKind) -> &'static str {
    match t {
        TransportKind::Bluetooth   => "bluetooth",
        TransportKind::WifiDirect  => "wifi_direct",
        TransportKind::LocalNetwork=> "local_network",
        TransportKind::QrBootstrap => "qr",
        TransportKind::SdCard      => "sdcard",
        TransportKind::UsbOtg      => "usb",
        TransportKind::LocalFile   => "file",
    }
}

fn parse_transport(s: &str) -> TransportKind {
    match s {
        "bluetooth"     => TransportKind::Bluetooth,
        "wifi_direct"   => TransportKind::WifiDirect,
        "local_network" => TransportKind::LocalNetwork,
        "qr"            => TransportKind::QrBootstrap,
        "sdcard"        => TransportKind::SdCard,
        "usb"           => TransportKind::UsbOtg,
        _               => TransportKind::LocalFile,
    }
}

fn sync_status_str(s: &SyncStatus) -> &'static str {
    match s {
        SyncStatus::Negotiating  => "negotiating",
        SyncStatus::Transferring => "transferring",
        SyncStatus::Verifying    => "verifying",
        SyncStatus::Completed    => "completed",
        SyncStatus::Failed       => "failed",
        SyncStatus::Aborted      => "aborted",
    }
}

fn parse_sync_status(s: &str) -> SyncStatus {
    match s {
        "negotiating"  => SyncStatus::Negotiating,
        "transferring" => SyncStatus::Transferring,
        "verifying"    => SyncStatus::Verifying,
        "completed"    => SyncStatus::Completed,
        "failed"       => SyncStatus::Failed,
        _              => SyncStatus::Aborted,
    }
}
