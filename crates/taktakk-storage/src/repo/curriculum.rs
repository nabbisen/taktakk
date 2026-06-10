//! Curriculum repository: modules, lessons, and lesson steps.

use sqlx::SqlitePool;

use taktakk_core::domain::curriculum::{Lesson, Module, ModuleStatus, ModuleVersion};
use taktakk_core::error::{CoreError, CoreResult};

// ── Modules ───────────────────────────────────────────────────────────────────

pub async fn upsert_module(pool: &SqlitePool, m: &Module) -> CoreResult<()> {
    let status_str = module_status_str(&m.status);
    sqlx::query(
        "INSERT INTO modules
             (module_id, category_id, title_key, description_key,
              version_major, version_minor, version_patch, status, estimated_minutes)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(module_id) DO UPDATE SET
             status            = excluded.status,
             version_major     = excluded.version_major,
             version_minor     = excluded.version_minor,
             version_patch     = excluded.version_patch,
             estimated_minutes = excluded.estimated_minutes",
    )
    .bind(&m.module_id)
    .bind(&m.category_id)
    .bind(&m.title_key)
    .bind(&m.description_key)
    .bind(m.version.major)
    .bind(m.version.minor)
    .bind(m.version.patch)
    .bind(status_str)
    .bind(m.estimated_minutes.map(|v| v as i64))
    .execute(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

pub async fn list_modules(pool: &SqlitePool) -> CoreResult<Vec<Module>> {
    type Row = (String, String, String, String, i64, i64, i64, String, Option<i64>);
    let rows = sqlx::query_as::<_, Row>(
        "SELECT module_id, category_id, title_key, description_key,
                version_major, version_minor, version_patch, status, estimated_minutes
         FROM modules
         ORDER BY category_id, module_id",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(rows.into_iter().map(row_to_module).collect())
}

pub async fn get_module(pool: &SqlitePool, module_id: &str) -> CoreResult<Option<Module>> {
    type Row = (String, String, String, String, i64, i64, i64, String, Option<i64>);
    let row = sqlx::query_as::<_, Row>(
        "SELECT module_id, category_id, title_key, description_key,
                version_major, version_minor, version_patch, status, estimated_minutes
         FROM modules WHERE module_id = ?",
    )
    .bind(module_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(row.map(row_to_module))
}

// ── Lessons ───────────────────────────────────────────────────────────────────

pub async fn upsert_lesson(pool: &SqlitePool, l: &Lesson) -> CoreResult<()> {
    sqlx::query(
        "INSERT INTO lessons (lesson_id, module_id, title_key, sort_order, step_count)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(lesson_id) DO UPDATE SET
             title_key  = excluded.title_key,
             sort_order = excluded.sort_order,
             step_count = excluded.step_count",
    )
    .bind(&l.lesson_id)
    .bind(&l.module_id)
    .bind(&l.title_key)
    .bind(l.sort_order)
    .bind(l.step_count)
    .execute(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

pub async fn list_lessons(pool: &SqlitePool, module_id: &str) -> CoreResult<Vec<Lesson>> {
    let rows = sqlx::query_as::<_, (String, String, String, i64, i64)>(
        "SELECT lesson_id, module_id, title_key, sort_order, step_count
         FROM lessons WHERE module_id = ? ORDER BY sort_order",
    )
    .bind(module_id)
    .fetch_all(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|(lesson_id, module_id, title_key, sort_order, step_count)| Lesson {
            lesson_id,
            module_id,
            title_key,
            sort_order: sort_order as u32,
            step_count: step_count as u32,
        })
        .collect())
}

pub async fn get_lesson(pool: &SqlitePool, lesson_id: &str) -> CoreResult<Option<Lesson>> {
    let row = sqlx::query_as::<_, (String, String, String, i64, i64)>(
        "SELECT lesson_id, module_id, title_key, sort_order, step_count
         FROM lessons WHERE lesson_id = ?",
    )
    .bind(lesson_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(row.map(|(lesson_id, module_id, title_key, sort_order, step_count)| Lesson {
        lesson_id,
        module_id,
        title_key,
        sort_order: sort_order as u32,
        step_count: step_count as u32,
    }))
}

// ── Wipe helpers ──────────────────────────────────────────────────────────────

pub async fn wipe_curriculum(pool: &SqlitePool) -> CoreResult<()> {
    sqlx::query("DELETE FROM lesson_steps").execute(pool).await
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    sqlx::query("DELETE FROM lessons").execute(pool).await
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    sqlx::query("DELETE FROM modules").execute(pool).await
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn module_status_str(s: &ModuleStatus) -> &'static str {
    match s {
        ModuleStatus::Available   => "available",
        ModuleStatus::Partial     => "partial",
        ModuleStatus::Quarantined => "quarantined",
        ModuleStatus::Disabled    => "disabled",
    }
}

fn parse_module_status(s: &str) -> ModuleStatus {
    match s {
        "available"   => ModuleStatus::Available,
        "partial"     => ModuleStatus::Partial,
        "quarantined" => ModuleStatus::Quarantined,
        _             => ModuleStatus::Disabled,
    }
}

type ModuleRow = (String, String, String, String, i64, i64, i64, String, Option<i64>);
fn row_to_module(r: ModuleRow) -> Module {
    let (module_id, category_id, title_key, description_key,
         major, minor, patch, status_str, est_mins) = r;
    Module {
        module_id,
        category_id,
        title_key,
        description_key,
        version: ModuleVersion::new(major as u16, minor as u16, patch as u16),
        status: parse_module_status(&status_str),
        estimated_minutes: est_mins.map(|v| v as u16),
    }
}
