//! Test utilities for domain layer tests.
//!
//! This module is gated behind `#[cfg(test)]` and provides:
//!
//! - [`MockRepository<T>`] — in-memory `Repository<T>` backed by a `Vec<T>`
//! - [`MockEventPublisher`] — collects published events for assertion
//! - [`MockCache<K, V>`] — `HashMap`-backed `Cache<K, V>`
//! - [`test_entity!`] — macro to create a minimal `Entity` impl for testing

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use async_trait::async_trait;

use crate::entity::{Entity, EntityId};
use crate::error::{DomainError, Result};
use crate::event::DomainEvent;
use crate::port::Cache;
use crate::repository::{Page, PaginatedRepository, Repository};

// ── MockRepository ────────────────────────────────────────────────────────────

/// An in-memory [`Repository<T>`] suitable for unit tests.
///
/// All entities are stored in a `Vec<T>` protected by a `Mutex` so the mock
/// can be shared across `Arc` clones in async test helpers.
///
/// # Example
///
/// ```rust
/// # use project_domain::tests::{MockRepository, make_test_entity};
/// # use project_domain::repository::Repository;
/// # tokio_test::block_on(async {
/// let repo: MockRepository<TestUser> = MockRepository::new();
/// let user = make_test_entity();
/// repo.save(&user).await.unwrap();
/// assert_eq!(repo.count_stored(), 1);
/// # })
/// ```
pub struct MockRepository<T: Entity + Clone> {
    store: Arc<Mutex<Vec<T>>>,
}

impl<T: Entity + Clone> MockRepository<T> {
    /// Create an empty repository.
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Seed the repository with pre-existing entities (useful for "given" setup).
    pub fn with_entities(entities: Vec<T>) -> Self {
        Self {
            store: Arc::new(Mutex::new(entities)),
        }
    }

    /// Return the number of entities currently stored.
    pub fn count_stored(&self) -> usize {
        self.store.lock().unwrap().len()
    }

    /// Assert that the store contains exactly `n` entities.
    ///
    /// # Panics
    ///
    /// Panics with a descriptive message if the count differs.
    pub fn assert_count(&self, n: usize) {
        let actual = self.count_stored();
        assert_eq!(
            actual, n,
            "MockRepository: expected {} entities, found {}",
            n, actual
        );
    }

    /// Return a snapshot clone of all stored entities.
    pub fn all_stored(&self) -> Vec<T> {
        self.store.lock().unwrap().clone()
    }
}

impl<T: Entity + Clone> Default for MockRepository<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T> Repository<T> for MockRepository<T>
where
    T: Entity + Clone,
{
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>> {
        let store = self.store.lock().unwrap();
        Ok(store.iter().find(|e| e.id() == id).cloned())
    }

    async fn save(&self, entity: &T) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        // Upsert: replace if present, push if absent.
        if let Some(pos) = store.iter().position(|e| e.id() == entity.id()) {
            store[pos] = entity.clone();
        } else {
            store.push(entity.clone());
        }
        Ok(())
    }

    async fn delete(&self, id: &T::Id) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        store.retain(|e| e.id() != id);
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<T>> {
        Ok(self.store.lock().unwrap().clone())
    }
}

#[async_trait]
impl<T> PaginatedRepository<T> for MockRepository<T>
where
    T: Entity + Clone,
{
    async fn find_page(&self, page: u32, size: u32) -> Result<Page<T>> {
        let store = self.store.lock().unwrap();
        let total = store.len() as u64;
        let start = (page as usize) * (size as usize);
        let items: Vec<T> = store.iter().skip(start).take(size as usize).cloned().collect();
        Ok(Page::new(items, total, page, size))
    }

    async fn count(&self) -> Result<u64> {
        Ok(self.store.lock().unwrap().len() as u64)
    }
}

// ── MockEventPublisher ────────────────────────────────────────────────────────

/// A [`crate::port::EventPublisher`] that collects all published events.
///
/// Use in tests to assert that specific events were emitted.
///
/// # Example
///
/// ```rust
/// # use project_domain::tests::MockEventPublisher;
/// # use project_domain::port::EventPublisher;
/// # use project_domain::event::DomainEvent;
/// # tokio_test::block_on(async {
/// let publisher = MockEventPublisher::new();
/// // publisher.publish(some_event).await.unwrap();
/// // publisher.assert_published(1);
/// # })
/// ```
pub struct MockEventPublisher {
    events: Arc<Mutex<Vec<Box<dyn std::any::Any + Send + Sync>>>>,
}

impl MockEventPublisher {
    /// Create a publisher with an empty event log.
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Assert that exactly `n` events have been published.
    ///
    /// # Panics
    ///
    /// Panics with a descriptive message if the count differs.
    pub fn assert_published(&self, n: usize) {
        let count = self.events.lock().unwrap().len();
        assert_eq!(
            count, n,
            "MockEventPublisher: expected {} events, found {}",
            n, count
        );
    }

    /// Return the total number of published events.
    pub fn published_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    /// Drain and return all collected type-erased event boxes.
    ///
    /// After calling this the internal log is empty.
    pub fn take_all_raw(&self) -> Vec<Box<dyn std::any::Any + Send + Sync>> {
        let mut events = self.events.lock().unwrap();
        std::mem::take(&mut *events)
    }

    /// Return `true` if no events have been published.
    pub fn is_empty(&self) -> bool {
        self.events.lock().unwrap().is_empty()
    }
}

impl Default for MockEventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

// We implement EventPublisher by boxing the event as Any.
// Tests can downcast using take_all_raw() if they need the concrete type.
#[async_trait]
impl crate::port::EventPublisher for MockEventPublisher {
    async fn publish<E: DomainEvent>(&self, event: E) -> Result<()> {
        let mut events = self.events.lock().unwrap();
        events.push(Box::new(event));
        Ok(())
    }
}

// ── MockCache ─────────────────────────────────────────────────────────────────

/// A `HashMap`-backed [`Cache<K, V>`] for tests.
///
/// Thread-safe via an inner `Mutex`. Does not support TTL; all entries
/// persist until explicitly deleted or the mock is dropped.
pub struct MockCache<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    store: Arc<Mutex<HashMap<K, V>>>,
}

impl<K, V> MockCache<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create an empty cache.
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Return the number of entries currently in the cache.
    pub fn size(&self) -> usize {
        self.store.lock().unwrap().len()
    }
}

impl<K, V> Default for MockCache<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for MockCache<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    async fn get(&self, key: &K) -> Result<Option<V>> {
        Ok(self.store.lock().unwrap().get(key).cloned())
    }

    async fn set(&self, key: K, value: V) -> Result<()> {
        self.store.lock().unwrap().insert(key, value);
        Ok(())
    }

    async fn delete(&self, key: &K) -> Result<()> {
        self.store.lock().unwrap().remove(key);
        Ok(())
    }
}

// ── test_entity! macro ────────────────────────────────────────────────────────

/// Create a minimal concrete `Entity` implementation for use in tests.
///
/// # Syntax
///
/// ```rust,ignore
/// test_entity!(MyTestEntity);
/// // or with a custom id type:
/// test_entity!(MyTestEntity, EntityId<MyTestEntity>);
/// ```
///
/// # What it generates
///
/// - A struct `$name { id: EntityId<$name>, created_at: SystemTime, updated_at: SystemTime }`
/// - `$name::new()` constructor
/// - `Entity for $name` implementation
/// - `Clone + Debug` derives
#[macro_export]
macro_rules! test_entity {
    ($name:ident) => {
        #[derive(Clone, Debug)]
        pub struct $name {
            id: $crate::entity::EntityId<$name>,
            created_at: ::std::time::SystemTime,
            updated_at: ::std::time::SystemTime,
        }

        impl $name {
            /// Create a new test entity with a random id.
            pub fn new() -> Self {
                let now = ::std::time::SystemTime::now();
                Self {
                    id: $crate::entity::EntityId::new(),
                    created_at: now,
                    updated_at: now,
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::entity::Entity for $name {
            type Id = $crate::entity::EntityId<$name>;

            fn id(&self) -> &Self::Id {
                &self.id
            }

            fn created_at(&self) -> ::std::time::SystemTime {
                self.created_at
            }

            fn updated_at(&self) -> ::std::time::SystemTime {
                self.updated_at
            }
        }
    };
}

// ── Test utilities ────────────────────────────────────────────────────────────

// Define a test entity using the macro for internal tests in this module.
test_entity!(TestUser);

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::port::EventPublisher;
    use uuid::Uuid;

    // ── MockRepository tests ──────────────────────────────────────────────────

    #[tokio::test]
    async fn mock_repo_save_and_find_by_id() {
        let repo: MockRepository<TestUser> = MockRepository::new();
        let user = TestUser::new();
        let id = *user.id();

        repo.save(&user).await.unwrap();
        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(*found.unwrap().id(), id);
    }

    #[tokio::test]
    async fn mock_repo_find_by_id_missing_returns_none() {
        let repo: MockRepository<TestUser> = MockRepository::new();
        let id: EntityId<TestUser> = EntityId::new();
        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn mock_repo_save_upserts() {
        let repo: MockRepository<TestUser> = MockRepository::new();
        let user = TestUser::new();
        repo.save(&user).await.unwrap();
        repo.save(&user).await.unwrap(); // second save of same id
        repo.assert_count(1); // still only one entity
    }

    #[tokio::test]
    async fn mock_repo_delete_removes_entity() {
        let repo: MockRepository<TestUser> = MockRepository::new();
        let user = TestUser::new();
        let id = *user.id();
        repo.save(&user).await.unwrap();
        repo.delete(&id).await.unwrap();
        assert!(repo.find_by_id(&id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn mock_repo_delete_nonexistent_is_ok() {
        let repo: MockRepository<TestUser> = MockRepository::new();
        let id: EntityId<TestUser> = EntityId::new();
        assert!(repo.delete(&id).await.is_ok());
    }

    #[tokio::test]
    async fn mock_repo_find_all() {
        let users: Vec<TestUser> = (0..5).map(|_| TestUser::new()).collect();
        let repo = MockRepository::with_entities(users);
        let all = repo.find_all().await.unwrap();
        assert_eq!(all.len(), 5);
    }

    #[tokio::test]
    async fn mock_repo_find_page() {
        let users: Vec<TestUser> = (0..15).map(|_| TestUser::new()).collect();
        let repo = MockRepository::with_entities(users);

        let page = repo.find_page(0, 10).await.unwrap();
        assert_eq!(page.items.len(), 10);
        assert_eq!(page.total, 15);
        assert!(page.has_next());

        let last = repo.find_page(1, 10).await.unwrap();
        assert_eq!(last.items.len(), 5);
        assert!(!last.has_next());
    }

    #[tokio::test]
    async fn mock_repo_count() {
        let repo: MockRepository<TestUser> = MockRepository::new();
        assert_eq!(repo.count().await.unwrap(), 0);
        repo.save(&TestUser::new()).await.unwrap();
        assert_eq!(repo.count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn mock_repo_exists() {
        let repo: MockRepository<TestUser> = MockRepository::new();
        let user = TestUser::new();
        let id = *user.id();
        assert!(!repo.exists(&id).await.unwrap());
        repo.save(&user).await.unwrap();
        assert!(repo.exists(&id).await.unwrap());
    }

    // ── MockEventPublisher tests ──────────────────────────────────────────────

    #[derive(Clone, Debug)]
    struct TestEvent {
        id: Uuid,
        agg_id: String,
        occurred_at: SystemTime,
    }

    impl TestEvent {
        fn new() -> Self {
            Self {
                id: Uuid::new_v4(),
                agg_id: "agg-1".to_string(),
                occurred_at: SystemTime::now(),
            }
        }
    }

    impl crate::event::DomainEvent for TestEvent {
        fn event_id(&self) -> Uuid { self.id }
        fn aggregate_id(&self) -> &str { &self.agg_id }
        fn occurred_at(&self) -> SystemTime { self.occurred_at }
        fn event_type(&self) -> &'static str { "test.happened" }
    }

    #[tokio::test]
    async fn mock_publisher_collects_events() {
        let publisher = MockEventPublisher::new();
        publisher.assert_published(0);
        assert!(publisher.is_empty());

        publisher.publish(TestEvent::new()).await.unwrap();
        publisher.assert_published(1);
        assert!(!publisher.is_empty());
    }

    #[tokio::test]
    async fn mock_publisher_take_all_raw_drains_store() {
        let publisher = MockEventPublisher::new();
        publisher.publish(TestEvent::new()).await.unwrap();
        publisher.publish(TestEvent::new()).await.unwrap();

        let events = publisher.take_all_raw();
        assert_eq!(events.len(), 2);
        publisher.assert_published(0); // log drained
    }

    // ── MockCache tests ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn mock_cache_set_and_get() {
        let cache: MockCache<String, String> = MockCache::new();
        cache.set("key".to_string(), "value".to_string()).await.unwrap();
        let got = cache.get(&"key".to_string()).await.unwrap();
        assert_eq!(got.as_deref(), Some("value"));
    }

    #[tokio::test]
    async fn mock_cache_miss_returns_none() {
        let cache: MockCache<String, i32> = MockCache::new();
        let got = cache.get(&"absent".to_string()).await.unwrap();
        assert!(got.is_none());
    }

    #[tokio::test]
    async fn mock_cache_delete_removes_key() {
        let cache: MockCache<String, i32> = MockCache::new();
        cache.set("k".to_string(), 42).await.unwrap();
        cache.delete(&"k".to_string()).await.unwrap();
        assert!(cache.get(&"k".to_string()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn mock_cache_contains() {
        let cache: MockCache<String, i32> = MockCache::new();
        assert!(!cache.contains(&"x".to_string()).await.unwrap());
        cache.set("x".to_string(), 1).await.unwrap();
        assert!(cache.contains(&"x".to_string()).await.unwrap());
    }

    #[tokio::test]
    async fn mock_cache_size() {
        let cache: MockCache<u32, u32> = MockCache::new();
        assert_eq!(cache.size(), 0);
        cache.set(1, 100).await.unwrap();
        cache.set(2, 200).await.unwrap();
        assert_eq!(cache.size(), 2);
        cache.delete(&1).await.unwrap();
        assert_eq!(cache.size(), 1);
    }

    // ── test_entity! macro ────────────────────────────────────────────────────

    #[test]
    fn test_entity_macro_creates_new_with_unique_id() {
        let a = TestUser::new();
        let b = TestUser::new();
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn test_entity_macro_implements_entity_trait() {
        let user = TestUser::new();
        // Accessing Entity trait methods:
        let _ = user.id();
        let _ = user.created_at();
        let _ = user.updated_at();
    }
}
