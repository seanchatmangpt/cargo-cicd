//! Repository port traits.
//!
//! Repositories abstract the persistence mechanism. The domain defines *what*
//! operations are needed; adapters (in the infrastructure layer) provide *how*
//! those operations are implemented (Postgres, SQLite, in-memory, etc.).
//!
//! # Traits
//!
//! | Trait | Purpose |
//! |-------|---------|
//! | [`Repository<T>`] | Basic CRUD operations for an entity |
//! | [`PaginatedRepository<T>`] | Extends `Repository<T>` with pagination |
//!
//! # Page
//!
//! [`Page<T>`] is the return type for paginated queries. It carries the items,
//! total count, and helpers for computing `has_next`, `total_pages`, etc.

use async_trait::async_trait;

use crate::entity::Entity;
use crate::error::Result;

// ── Repository ────────────────────────────────────────────────────────────────

/// Core CRUD repository for a domain entity `T`.
///
/// Every entity that needs to be persisted should have a corresponding
/// `Repository<T>` implementation provided by the infrastructure layer and
/// injected into application services through this trait.
///
/// All methods are async to support both blocking and non-blocking adapters.
/// Implementors must be `Send + Sync` for use in async runtimes.
///
/// # Contracts
///
/// - `find_by_id` returns `Ok(None)` (not an error) when the entity does not exist.
/// - `save` performs an upsert — insert if absent, update if present.
/// - `delete` is idempotent — deleting a non-existent entity returns `Ok(())`.
/// - `find_all` may be expensive on large tables; prefer pagination for production.
#[async_trait]
pub trait Repository<T: Entity>: Send + Sync + 'static {
    /// Look up an entity by its primary identifier.
    ///
    /// Returns `Ok(None)` if no entity with that id exists.
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>>;

    /// Persist the entity (insert or update).
    ///
    /// After this call the entity is durable according to the adapter's
    /// consistency guarantees (e.g. fsync, transaction commit).
    async fn save(&self, entity: &T) -> Result<()>;

    /// Remove the entity with the given id.
    ///
    /// Returns `Ok(())` if the entity did not exist (idempotent).
    async fn delete(&self, id: &T::Id) -> Result<()>;

    /// Return every entity of type `T` managed by this repository.
    ///
    /// Use with caution in production — prefer [`PaginatedRepository::find_page`]
    /// to avoid loading unbounded result sets into memory.
    async fn find_all(&self) -> Result<Vec<T>>;

    /// Return `true` if an entity with this id exists.
    ///
    /// The default implementation calls [`find_by_id`]; adapters may override
    /// with a cheaper EXISTS query.
    ///
    /// [`find_by_id`]: Repository::find_by_id
    async fn exists(&self, id: &T::Id) -> Result<bool> {
        Ok(self.find_by_id(id).await?.is_some())
    }
}

// ── PaginatedRepository ───────────────────────────────────────────────────────

/// Extension of [`Repository<T>`] that adds cursor-free offset pagination.
///
/// Offset pagination is simple and widely supported, but may be inconsistent
/// under concurrent inserts. For high-throughput systems consider keyset
/// pagination at the adapter level.
#[async_trait]
pub trait PaginatedRepository<T: Entity>: Repository<T> {
    /// Fetch one page of results.
    ///
    /// - `page` is **zero-based** (page 0 is the first page).
    /// - `size` is the maximum number of items per page.
    ///
    /// Returns a [`Page<T>`] that includes the total count for computing
    /// page navigation.
    async fn find_page(&self, page: u32, size: u32) -> Result<Page<T>>;

    /// Return the total number of entities managed by this repository.
    async fn count(&self) -> Result<u64>;
}

// ── Page ──────────────────────────────────────────────────────────────────────

/// A single page of query results with pagination metadata.
///
/// # Example
///
/// ```rust
/// use project_domain::repository::Page;
///
/// let page: Page<String> = Page {
///     items: vec!["a".to_string(), "b".to_string()],
///     total: 42,
///     page: 0,
///     size: 2,
/// };
///
/// assert!(!page.is_empty());
/// assert!(page.has_next());
/// assert_eq!(page.total_pages(), 21);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Page<T> {
    /// The items on this page.
    pub items: Vec<T>,
    /// Total number of items across all pages.
    pub total: u64,
    /// Zero-based page index.
    pub page: u32,
    /// Maximum items per page requested.
    pub size: u32,
}

impl<T> Page<T> {
    /// Construct a new `Page`.
    pub fn new(items: Vec<T>, total: u64, page: u32, size: u32) -> Self {
        Self {
            items,
            total,
            page,
            size,
        }
    }

    /// Construct an empty first page (useful as a fallback).
    pub fn empty(size: u32) -> Self {
        Self {
            items: Vec::new(),
            total: 0,
            page: 0,
            size,
        }
    }

    /// Return `true` if there are no items on this page.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Return `true` if there is a subsequent page.
    pub fn has_next(&self) -> bool {
        let offset = (self.page as u64) * (self.size as u64);
        offset + (self.items.len() as u64) < self.total
    }

    /// Return `true` if this is not the first page.
    pub fn has_previous(&self) -> bool {
        self.page > 0
    }

    /// Total number of pages given the current page size.
    ///
    /// Returns 0 if `size` is 0 or `total` is 0.
    pub fn total_pages(&self) -> u64 {
        if self.size == 0 || self.total == 0 {
            return 0;
        }
        (self.total + self.size as u64 - 1) / self.size as u64
    }

    /// The number of items on this page.
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl<T> IntoIterator for Page<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_page(items: usize, total: u64, page: u32, size: u32) -> Page<i32> {
        Page::new((0..items as i32).collect(), total, page, size)
    }

    #[test]
    fn page_has_next_true_when_more_items_remain() {
        let p = make_page(10, 25, 0, 10);
        assert!(p.has_next(), "page 0 of 25 with size 10 should have next");
    }

    #[test]
    fn page_has_next_false_on_last_page() {
        let p = make_page(5, 15, 1, 10);
        // page 1 (second page), items 10-14 out of 15 total
        assert!(!p.has_next());
    }

    #[test]
    fn page_has_previous() {
        let first = make_page(10, 20, 0, 10);
        let second = make_page(10, 20, 1, 10);
        assert!(!first.has_previous());
        assert!(second.has_previous());
    }

    #[test]
    fn page_total_pages() {
        assert_eq!(make_page(10, 25, 0, 10).total_pages(), 3);
        assert_eq!(make_page(10, 20, 0, 10).total_pages(), 2);
        assert_eq!(make_page(0, 0, 0, 10).total_pages(), 0);
        assert_eq!(make_page(0, 1, 0, 0).total_pages(), 0); // size=0 guard
    }

    #[test]
    fn page_is_empty() {
        assert!(Page::<i32>::empty(10).is_empty());
        assert!(!make_page(1, 1, 0, 10).is_empty());
    }

    #[test]
    fn page_into_iter() {
        let page = Page::new(vec![1, 2, 3], 3, 0, 10);
        let collected: Vec<i32> = page.into_iter().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn page_len() {
        let p = make_page(7, 100, 0, 10);
        assert_eq!(p.len(), 7);
    }
}
