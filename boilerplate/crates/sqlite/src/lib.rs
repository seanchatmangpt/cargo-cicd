//! SQLite persistence layer for the project boilerplate.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use project_sqlite::{Database, JsonRepository, run_migrations};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // 1. Open a database (file or :memory:).
//!     let db = Database::connect("sqlite::memory:").await?;
//!
//!     // 2. Apply schema migrations.
//!     run_migrations(db.pool()).await?;
//!
//!     // 3. Create a typed repository.
//!     let repo: JsonRepository<serde_json::Value> =
//!         JsonRepository::new(db.pool().clone(), "items");
//!
//!     // 4. Persist and retrieve.
//!     repo.save("id-1", &serde_json::json!({"hello": "world"})).await?;
//!     let val = repo.find_by_id("id-1").await?;
//!     println!("{val:?}");
//!
//!     Ok(())
//! }
//! ```

pub mod connection;
pub mod error;
pub mod migrations;
pub mod repository;

// ---------------------------------------------------------------------------
// Re-exports — the items callers need most often
// ---------------------------------------------------------------------------

pub use connection::{create_in_memory, Database, DatabaseUrl};
pub use error::{Result, StorageError};
pub use migrations::run_migrations;
pub use repository::{JsonRepository, Repository};
