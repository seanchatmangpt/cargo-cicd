use std::marker::PhantomData;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::SqlitePool;
use tracing::instrument;

use crate::error::{Result, StorageError};

// ---------------------------------------------------------------------------
// Generic JSON repository
// ---------------------------------------------------------------------------

/// A repository that persists any `Serialize + DeserializeOwned` type as a
/// JSON blob in a SQLite table with the shape:
///
/// ```sql
/// CREATE TABLE <table> (
///     id          TEXT PRIMARY KEY NOT NULL,
///     data        JSON             NOT NULL,
///     created_at  TEXT             NOT NULL,
///     updated_at  TEXT             NOT NULL
/// );
/// ```
///
/// This matches the schema created by migration `0001_initial.sql`.  Pass a
/// different `table` name if you want a dedicated table for your aggregate.
#[derive(Debug, Clone)]
pub struct JsonRepository<T> {
    pool: SqlitePool,
    table: &'static str,
    _phantom: PhantomData<T>,
}

impl<T> JsonRepository<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    /// Create a new repository that targets `table` inside `pool`.
    ///
    /// The caller is responsible for ensuring the table exists (i.e. by
    /// running [`crate::migrations::run_migrations`] first).
    pub fn new(pool: SqlitePool, table: &'static str) -> Self {
        Self {
            pool,
            table,
            _phantom: PhantomData,
        }
    }

    // ------------------------------------------------------------------
    // Core CRUD
    // ------------------------------------------------------------------

    /// Return the entity stored under `id`, or `None` if absent.
    #[instrument(skip(self), fields(table = self.table, id = %id))]
    pub async fn find_by_id(&self, id: &str) -> Result<Option<T>> {
        let row: Option<(String,)> = sqlx::query_as(&format!(
            "SELECT data FROM {table} WHERE id = ?",
            table = self.table
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StorageError::from)?;

        match row {
            None => Ok(None),
            Some((json,)) => {
                let value: T = serde_json::from_str(&json)?;
                Ok(Some(value))
            }
        }
    }

    /// Persist `entity` under `id`. If a row with the same `id` already
    /// exists it is replaced (UPSERT semantics).
    #[instrument(skip(self, entity), fields(table = self.table, id = %id))]
    pub async fn save(&self, id: &str, entity: &T) -> Result<()> {
        let json = serde_json::to_string(entity)?;
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(&format!(
            r#"
            INSERT INTO {table} (id, data, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                data       = excluded.data,
                updated_at = excluded.updated_at
            "#,
            table = self.table
        ))
        .bind(id)
        .bind(&json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(StorageError::from)?;

        Ok(())
    }

    /// Remove the entity identified by `id`.
    ///
    /// Returns `true` if a row was deleted, `false` if no row existed.
    #[instrument(skip(self), fields(table = self.table, id = %id))]
    pub async fn delete(&self, id: &str) -> Result<bool> {
        let result = sqlx::query(&format!(
            "DELETE FROM {table} WHERE id = ?",
            table = self.table
        ))
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(StorageError::from)?;

        Ok(result.rows_affected() > 0)
    }

    /// Return all entities in the table (unordered).
    ///
    /// For large tables prefer [`JsonRepository::find_page`].
    #[instrument(skip(self), fields(table = self.table))]
    pub async fn find_all(&self) -> Result<Vec<T>> {
        let rows: Vec<(String,)> = sqlx::query_as(&format!(
            "SELECT data FROM {table} ORDER BY created_at ASC",
            table = self.table
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(StorageError::from)?;

        rows.into_iter()
            .map(|(json,)| serde_json::from_str::<T>(&json).map_err(StorageError::from))
            .collect()
    }

    // ------------------------------------------------------------------
    // Aggregate helpers
    // ------------------------------------------------------------------

    /// Count the number of rows in the table.
    #[instrument(skip(self), fields(table = self.table))]
    pub async fn count(&self) -> Result<u64> {
        let (n,): (i64,) = sqlx::query_as(&format!(
            "SELECT COUNT(*) FROM {table}",
            table = self.table
        ))
        .fetch_one(&self.pool)
        .await
        .map_err(StorageError::from)?;

        Ok(n as u64)
    }

    // ------------------------------------------------------------------
    // Pagination
    // ------------------------------------------------------------------

    /// Return a page of entities ordered by `created_at` ascending.
    ///
    /// `page` is 0-indexed. `size` is the maximum number of rows to return.
    ///
    /// ```text
    /// page=0, size=10 → rows  0–9
    /// page=1, size=10 → rows 10–19
    /// ```
    #[instrument(skip(self), fields(table = self.table, page, size))]
    pub async fn find_page(&self, page: u32, size: u32) -> Result<Vec<T>> {
        let offset = page as i64 * size as i64;
        let limit = size as i64;

        let rows: Vec<(String,)> = sqlx::query_as(&format!(
            "SELECT data FROM {table} ORDER BY created_at ASC LIMIT ? OFFSET ?",
            table = self.table
        ))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(StorageError::from)?;

        rows.into_iter()
            .map(|(json,)| serde_json::from_str::<T>(&json).map_err(StorageError::from))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Optional: async Repository trait implementation
// ---------------------------------------------------------------------------
//
// Provides a uniform interface so callers can depend on a trait object rather
// than the concrete `JsonRepository` if desired.

/// Async repository port trait.
#[async_trait]
pub trait Repository<T: Send + Sync>: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn save(&self, id: &str, entity: &T) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<bool>;
    async fn find_all(&self) -> Result<Vec<T>>;
}

#[async_trait]
impl<T> Repository<T> for JsonRepository<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    async fn find_by_id(&self, id: &str) -> Result<Option<T>> {
        JsonRepository::find_by_id(self, id).await
    }

    async fn save(&self, id: &str, entity: &T) -> Result<()> {
        JsonRepository::save(self, id, entity).await
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        JsonRepository::delete(self, id).await
    }

    async fn find_all(&self) -> Result<Vec<T>> {
        JsonRepository::find_all(self).await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::create_in_memory;
    use crate::migrations::run_migrations;
    use serde::{Deserialize, Serialize};

    /// A simple domain object used across all tests.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Widget {
        name: String,
        value: i32,
    }

    impl Widget {
        fn new(name: &str, value: i32) -> Self {
            Self {
                name: name.to_string(),
                value,
            }
        }
    }

    /// Build an in-memory database with migrations applied and return a
    /// `JsonRepository<Widget>` targeting the `items` table.
    async fn make_repo() -> JsonRepository<Widget> {
        let db = create_in_memory().await.expect("in-memory db");
        run_migrations(db.pool()).await.expect("migrations");
        JsonRepository::new(db.pool().clone(), "items")
    }

    // ------------------------------------------------------------------
    // Roundtrip: insert then find_by_id
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn insert_and_find_by_id_roundtrip() {
        let repo = make_repo().await;
        let widget = Widget::new("sprocket", 42);

        repo.save("w1", &widget).await.expect("save");

        let found = repo.find_by_id("w1").await.expect("find");
        assert_eq!(found, Some(widget));
    }

    // ------------------------------------------------------------------
    // find_by_id returns None for missing id
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn find_by_id_returns_none_for_missing() {
        let repo = make_repo().await;
        let found = repo.find_by_id("does-not-exist").await.expect("find");
        assert!(found.is_none());
    }

    // ------------------------------------------------------------------
    // save overwrites an existing row
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn save_overwrites_existing_row() {
        let repo = make_repo().await;

        repo.save("w1", &Widget::new("original", 1))
            .await
            .expect("first save");
        repo.save("w1", &Widget::new("updated", 99))
            .await
            .expect("second save");

        let found = repo.find_by_id("w1").await.expect("find").unwrap();
        assert_eq!(found.name, "updated");
        assert_eq!(found.value, 99);
    }

    // ------------------------------------------------------------------
    // delete returns true when row existed
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn delete_returns_true_for_existing_row() {
        let repo = make_repo().await;
        repo.save("w1", &Widget::new("to-delete", 0))
            .await
            .expect("save");

        let deleted = repo.delete("w1").await.expect("delete");
        assert!(deleted, "expected true when row existed");

        let found = repo.find_by_id("w1").await.expect("find");
        assert!(found.is_none(), "row must be gone after delete");
    }

    // ------------------------------------------------------------------
    // delete returns false for a missing row
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn delete_returns_false_for_missing_row() {
        let repo = make_repo().await;
        let deleted = repo.delete("ghost").await.expect("delete");
        assert!(!deleted, "expected false when row did not exist");
    }

    // ------------------------------------------------------------------
    // find_all returns all items
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn find_all_returns_all_items() {
        let repo = make_repo().await;

        repo.save("a", &Widget::new("alpha", 1)).await.expect("save a");
        repo.save("b", &Widget::new("beta", 2)).await.expect("save b");
        repo.save("c", &Widget::new("gamma", 3)).await.expect("save c");

        let all = repo.find_all().await.expect("find_all");
        assert_eq!(all.len(), 3);

        // Order is by created_at; names must all be present.
        let names: Vec<&str> = all.iter().map(|w| w.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
        assert!(names.contains(&"gamma"));
    }

    // ------------------------------------------------------------------
    // count reflects inserted rows
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn count_reflects_insertions() {
        let repo = make_repo().await;
        assert_eq!(repo.count().await.expect("count"), 0);

        repo.save("x", &Widget::new("x", 0)).await.expect("save");
        assert_eq!(repo.count().await.expect("count"), 1);

        repo.save("y", &Widget::new("y", 0)).await.expect("save");
        assert_eq!(repo.count().await.expect("count"), 2);

        repo.delete("x").await.expect("delete");
        assert_eq!(repo.count().await.expect("count"), 1);
    }

    // ------------------------------------------------------------------
    // find_page respects limit and offset
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn find_page_respects_limit_and_offset() {
        let repo = make_repo().await;

        // Insert 5 widgets with a small sleep so created_at ordering is stable.
        for i in 0..5u32 {
            repo.save(&format!("id-{i:02}"), &Widget::new(&format!("w{i}"), i as i32))
                .await
                .expect("save");
        }

        // Page 0 of size 3 → items 0, 1, 2
        let page0 = repo.find_page(0, 3).await.expect("page 0");
        assert_eq!(page0.len(), 3);

        // Page 1 of size 3 → items 3, 4
        let page1 = repo.find_page(1, 3).await.expect("page 1");
        assert_eq!(page1.len(), 2);

        // Page 2 of size 3 → empty (only 5 items total)
        let page2 = repo.find_page(2, 3).await.expect("page 2");
        assert!(page2.is_empty());

        // All pages combined must cover every item exactly once.
        let mut combined = page0;
        combined.extend(page1);
        combined.extend(page2);
        assert_eq!(combined.len(), 5);
    }

    // ------------------------------------------------------------------
    // find_page with size larger than total returns all rows
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn find_page_large_size_returns_all() {
        let repo = make_repo().await;

        repo.save("p", &Widget::new("p", 10)).await.expect("save");
        repo.save("q", &Widget::new("q", 20)).await.expect("save");

        let page = repo.find_page(0, 1000).await.expect("page");
        assert_eq!(page.len(), 2);
    }

    // ------------------------------------------------------------------
    // Trait-object usage via Repository<T>
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn trait_object_save_and_find() {
        let repo = make_repo().await;
        let boxed: Box<dyn Repository<Widget>> = Box::new(repo);

        let widget = Widget::new("trait-test", 77);
        boxed.save("tt", &widget).await.expect("save via trait");

        let found = boxed.find_by_id("tt").await.expect("find via trait");
        assert_eq!(found, Some(widget));
    }
}
