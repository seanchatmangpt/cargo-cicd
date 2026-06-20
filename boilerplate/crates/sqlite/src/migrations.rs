use tracing::instrument;

use crate::error::{Result, StorageError};

// ---------------------------------------------------------------------------
// Embedded migration SQL
// ---------------------------------------------------------------------------
//
// These constants are the canonical source for the schema. The identical text
// is also written to `migrations/0001_initial.sql` and
// `migrations/0002_events.sql` so that the `sqlx migrate` CLI and the
// `sqlx::migrate!` macro can both consume them.  The `apply_embedded`
// function below drives the migrations programmatically, which is useful when
// the binary is deployed without the migration files on disk (e.g. inside a
// container image that only ships the binary).

/// Generic JSON-blob entity store. Any domain aggregate can be persisted here
/// by serialising its value to a JSON column.
const MIGRATION_0001_INITIAL: &str = r#"
CREATE TABLE IF NOT EXISTS items (
    id          TEXT PRIMARY KEY NOT NULL,
    data        JSON             NOT NULL,
    created_at  TEXT             NOT NULL,
    updated_at  TEXT             NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_items_created_at ON items (created_at);
"#;

/// Domain-event store for event-sourced aggregates.
const MIGRATION_0002_EVENTS: &str = r#"
CREATE TABLE IF NOT EXISTS events (
    id            TEXT    PRIMARY KEY NOT NULL,
    aggregate_id  TEXT    NOT NULL,
    event_type    TEXT    NOT NULL,
    payload       JSON    NOT NULL,
    occurred_at   TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_events_aggregate_id ON events (aggregate_id);
CREATE INDEX IF NOT EXISTS idx_events_occurred_at  ON events (occurred_at);
CREATE INDEX IF NOT EXISTS idx_events_event_type   ON events (event_type);
"#;

/// Migration version tracking table. Created before any domain migrations so
/// that `apply_embedded` is idempotent.
const MIGRATION_SCHEMA_VERSIONS: &str = r#"
CREATE TABLE IF NOT EXISTS _schema_versions (
    version     INTEGER  PRIMARY KEY NOT NULL,
    applied_at  TEXT     NOT NULL
);
"#;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Apply all embedded migrations to `pool` in version order.
///
/// Each migration is wrapped in its own transaction. If a version has already
/// been recorded in `_schema_versions` it is skipped, making the function
/// safe to call on every application start-up.
#[instrument(skip(pool))]
pub async fn run_migrations(pool: &sqlx::SqlitePool) -> Result<()> {
    // Bootstrap the version-tracking table first.
    sqlx::query(MIGRATION_SCHEMA_VERSIONS)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    let migrations: &[(i64, &str)] = &[
        (1, MIGRATION_0001_INITIAL),
        (2, MIGRATION_0002_EVENTS),
    ];

    for (version, sql) in migrations {
        apply_if_needed(pool, *version, sql).await?;
    }

    tracing::info!("all migrations applied");
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn apply_if_needed(
    pool: &sqlx::SqlitePool,
    version: i64,
    sql: &str,
) -> Result<()> {
    // Check whether this version has already been applied.
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM _schema_versions WHERE version = ?")
            .bind(version)
            .fetch_one(pool)
            .await
            .map_err(|e| StorageError::Connection { source: e })?;

    if row.0 > 0 {
        tracing::debug!(version, "migration already applied — skipping");
        return Ok(());
    }

    tracing::info!(version, "applying migration");

    // Execute the migration DDL inside an explicit transaction.
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    // sqlx::query does not support multiple statements in a single call; we
    // split on `;` and execute each statement individually.
    for statement in sql.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }
        sqlx::query(trimmed)
            .execute(&mut *tx)
            .await
            .map_err(|e| StorageError::Connection { source: e })?;
    }

    // Record the applied version.
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO _schema_versions (version, applied_at) VALUES (?, ?)",
    )
    .bind(version)
    .bind(&now)
    .execute(&mut *tx)
    .await
    .map_err(|e| StorageError::Connection { source: e })?;

    tx.commit()
        .await
        .map_err(|e| StorageError::Connection { source: e })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::create_in_memory;

    #[tokio::test]
    async fn migrations_are_idempotent() {
        let db = create_in_memory().await.expect("in-memory db");
        // Run twice — second run must not error.
        run_migrations(db.pool()).await.expect("first run");
        run_migrations(db.pool()).await.expect("second run (idempotent)");

        // Both tables must exist.
        let (items_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM items")
                .fetch_one(db.pool())
                .await
                .expect("items table exists");
        assert_eq!(items_count, 0);

        let (events_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM events")
                .fetch_one(db.pool())
                .await
                .expect("events table exists");
        assert_eq!(events_count, 0);
    }

    #[tokio::test]
    async fn version_table_tracks_applied_migrations() {
        let db = create_in_memory().await.expect("in-memory db");
        run_migrations(db.pool()).await.expect("migrations");

        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM _schema_versions")
                .fetch_one(db.pool())
                .await
                .expect("version table");
        assert_eq!(count, 2, "two migrations must be recorded");
    }
}
