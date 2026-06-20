use std::path::{Path, PathBuf};

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tracing::instrument;

use crate::error::{Result, StorageError};

/// Wraps a SQLite connection pool and exposes ergonomic constructors.
#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Open (or create) a database at the given URL.
    ///
    /// Accepts SQLite DSNs such as:
    /// - `"sqlite::memory:"`          — transient in-process database
    /// - `"sqlite:./data.db"`         — relative file
    /// - `"sqlite:///abs/path/to.db"` — absolute file
    #[instrument(skip_all, fields(url = %url))]
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(16)
            .connect(url)
            .await
            .map_err(|e| StorageError::Connection { source: e })?;

        Ok(Self { pool })
    }

    /// Open (or create) a database at the given filesystem path.
    ///
    /// The `create_if_missing` SQLite pragma is set automatically.
    #[instrument(skip_all, fields(path = %path.display()))]
    pub async fn connect_file(path: &Path) -> Result<Self> {
        let url = DatabaseUrl::builder()
            .path(path.to_path_buf())
            .create_if_missing(true)
            .to_connection_string();

        Self::connect(&url).await
    }

    /// Return a reference to the underlying connection pool.
    ///
    /// Use this to hand the pool to repositories or run raw queries.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Gracefully close all pooled connections.
    pub async fn close(self) {
        self.pool.close().await;
    }
}

/// Create a transient in-memory database. Useful in tests.
pub async fn create_in_memory() -> Result<Database> {
    Database::connect("sqlite::memory:").await
}

// ---------------------------------------------------------------------------
// DatabaseUrl builder
// ---------------------------------------------------------------------------

/// Builder for constructing a SQLite connection URL from structured fields.
#[derive(Debug, Default)]
pub struct DatabaseUrl {
    path: Option<PathBuf>,
    create_if_missing: bool,
    read_only: bool,
    /// Maximum number of milliseconds to wait for a connection.
    busy_timeout_ms: Option<u64>,
}

impl DatabaseUrl {
    /// Start building a URL.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Set the filesystem path for the database file.
    ///
    /// If `None`, the URL will produce an in-memory database.
    pub fn path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    /// Whether to create the database file if it does not yet exist
    /// (default: `false`).
    pub fn create_if_missing(mut self, yes: bool) -> Self {
        self.create_if_missing = yes;
        self
    }

    /// Open the database in read-only mode (default: `false`).
    pub fn read_only(mut self, yes: bool) -> Self {
        self.read_only = yes;
        self
    }

    /// Set the busy-timeout (milliseconds). SQLite returns `SQLITE_BUSY`
    /// instead of blocking indefinitely when writers contend.
    pub fn busy_timeout_ms(mut self, ms: u64) -> Self {
        self.busy_timeout_ms = Some(ms);
        self
    }

    /// Finalise and return the built URL as a `String`.
    pub fn to_connection_string(&self) -> String {
        let base = match &self.path {
            None => "sqlite::memory:".to_string(),
            Some(p) => {
                let display = p.display();
                format!("sqlite://{display}")
            }
        };

        let mut params: Vec<String> = Vec::new();

        if self.create_if_missing {
            params.push("mode=rwc".into());
        }
        if self.read_only {
            params.push("mode=ro".into());
        }
        if let Some(ms) = self.busy_timeout_ms {
            params.push(format!("_busy_timeout={ms}"));
        }

        if params.is_empty() {
            base
        } else {
            format!("{}?{}", base, params.join("&"))
        }
    }
}

// Keep the old `to_string` name as an alias so call-sites are not surprised.
impl DatabaseUrl {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.to_connection_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_database_connects() {
        let db = create_in_memory().await.expect("connect to :memory:");
        // A trivial query confirms the pool is live.
        let row: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(db.pool())
            .await
            .expect("select 1");
        assert_eq!(row.0, 1);
        db.close().await;
    }

    #[test]
    fn url_builder_memory() {
        let url = DatabaseUrl::builder().to_connection_string();
        assert_eq!(url, "sqlite::memory:");
    }

    #[test]
    fn url_builder_file_with_flags() {
        let url = DatabaseUrl::builder()
            .path(PathBuf::from("/tmp/test.db"))
            .create_if_missing(true)
            .busy_timeout_ms(5000)
            .to_connection_string();
        assert!(url.contains("sqlite://"));
        assert!(url.contains("mode=rwc"));
        assert!(url.contains("_busy_timeout=5000"));
    }
}
