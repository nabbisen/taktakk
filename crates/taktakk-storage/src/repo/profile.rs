//! Local profile repository.

use sqlx::SqlitePool;

use taktakk_core::domain::profile::LocalProfile;
use taktakk_core::error::{CoreError, CoreResult};

/// Save or update a local profile.
pub async fn save(pool: &SqlitePool, p: &LocalProfile) -> CoreResult<()> {
    sqlx::query(
        "INSERT INTO local_profiles
             (profile_id, display_alias, locale, created_at, last_active_at)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(profile_id) DO UPDATE SET
             display_alias  = excluded.display_alias,
             locale         = excluded.locale,
             last_active_at = excluded.last_active_at",
    )
    .bind(&p.profile_id)
    .bind(&p.display_alias)
    .bind(&p.locale)
    .bind(p.created_at)
    .bind(p.last_active_at)
    .execute(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

/// Retrieve a profile by its ID.
pub async fn get(pool: &SqlitePool, profile_id: &str) -> CoreResult<Option<LocalProfile>> {
    let row = sqlx::query_as::<_, (String, Option<String>, Option<String>, i64, Option<i64>)>(
        "SELECT profile_id, display_alias, locale, created_at, last_active_at
         FROM local_profiles
         WHERE profile_id = ?",
    )
    .bind(profile_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;

    Ok(row.map(|(profile_id, display_alias, locale, created_at, last_active_at)| {
        LocalProfile { profile_id, display_alias, locale, created_at, last_active_at }
    }))
}

/// Return the most-recently-active profile (if any).
pub async fn get_active(pool: &SqlitePool) -> CoreResult<Option<LocalProfile>> {
    let row = sqlx::query_as::<_, (String, Option<String>, Option<String>, i64, Option<i64>)>(
        "SELECT profile_id, display_alias, locale, created_at, last_active_at
         FROM local_profiles
         ORDER BY COALESCE(last_active_at, created_at) DESC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;

    Ok(row.map(|(profile_id, display_alias, locale, created_at, last_active_at)| {
        LocalProfile { profile_id, display_alias, locale, created_at, last_active_at }
    }))
}

/// Touch `last_active_at` for an existing profile.
pub async fn touch(pool: &SqlitePool, profile_id: &str, now: i64) -> CoreResult<()> {
    sqlx::query("UPDATE local_profiles SET last_active_at = ? WHERE profile_id = ?")
        .bind(now)
        .bind(profile_id)
        .execute(pool)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

/// List all profiles ordered by most-recently active.
pub async fn list(pool: &SqlitePool) -> CoreResult<Vec<LocalProfile>> {
    let rows = sqlx::query_as::<_, (String, Option<String>, Option<String>, i64, Option<i64>)>(
        "SELECT profile_id, display_alias, locale, created_at, last_active_at
         FROM local_profiles
         ORDER BY COALESCE(last_active_at, created_at) DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|(profile_id, display_alias, locale, created_at, last_active_at)| {
            LocalProfile { profile_id, display_alias, locale, created_at, last_active_at }
        })
        .collect())
}

/// Delete all profiles — used during hard wipe.
pub async fn wipe_all(pool: &SqlitePool) -> CoreResult<()> {
    sqlx::query("DELETE FROM local_profiles")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}
