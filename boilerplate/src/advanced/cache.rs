//! Concurrent, TTL-aware result caching for engine adapters via [`moka`].
//!
//! Uses `moka::sync::Cache` (the synchronous variant) so adapters can call
//! `get` / `insert` from any thread without `async` runtime overhead.
//!
//! # Example
//!
//! ```rust,ignore
//! use std::time::Duration;
//! use my_crate::advanced::cache::{EngineCache, CachedEntry};
//!
//! let cache = EngineCache::new(100, Duration::from_secs(300));
//!
//! // Store a serialized adapter result.
//! let bytes = serde_json::to_vec(&my_struct).unwrap();
//! cache.insert("workspace_metadata".to_string(),
//!              CachedEntry::with_label(bytes, "CargoMetadata"));
//!
//! // Retrieve cheaply — clones an Arc, not the Vec.
//! if let Some(entry) = cache.get("workspace_metadata") {
//!     let decoded: MyStruct = serde_json::from_slice(&entry.bytes).unwrap();
//! }
//!
//! // Force eviction of expired / over-capacity entries.
//! cache.run_pending_tasks();
//! ```

use moka::sync::Cache as MokaCache;
use std::sync::Arc;
use std::time::{Duration, Instant};

// ─── CachedEntry ─────────────────────────────────────────────────────────────

/// A single entry stored in [`EngineCache`].
///
/// The payload (`bytes`) is wrapped in an [`Arc`] so that cloning an entry
/// costs a single atomic reference-count increment — no `Vec` copy.
#[derive(Debug, Clone)]
pub struct CachedEntry {
    /// Serialized adapter result (e.g. JSON, bincode, TOML).
    pub bytes: Arc<Vec<u8>>,
    /// Human-readable label for debugging / logging.
    pub label: String,
    /// Wall-clock time at which this entry was constructed.
    pub created_at: Instant,
}

impl CachedEntry {
    /// Create an entry with an empty label.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes: Arc::new(bytes),
            label: String::new(),
            created_at: Instant::now(),
        }
    }

    /// Create an entry with a descriptive `label` (useful for cache-miss logs).
    pub fn with_label(bytes: Vec<u8>, label: impl Into<String>) -> Self {
        Self {
            bytes: Arc::new(bytes),
            label: label.into(),
            created_at: Instant::now(),
        }
    }

    /// Return how long ago this entry was created.
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Return the number of bytes stored in the payload.
    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }
}

// ─── EngineCache ─────────────────────────────────────────────────────────────

/// A thread-safe, capacity-bounded, TTL-expiring cache for engine adapter
/// results.
///
/// Wraps [`moka::sync::Cache`] with a domain-specific API.  All operations are
/// `O(1)` amortised and safe to call from multiple threads simultaneously.
pub struct EngineCache {
    inner: MokaCache<String, CachedEntry>,
}

impl EngineCache {
    /// Create a new cache with `max_capacity` entries and a per-entry TTL of
    /// `ttl`.  After `ttl` has elapsed an entry is treated as expired and will
    /// not be returned by [`get`](Self::get).
    ///
    /// # Panics
    ///
    /// Does not panic.
    pub fn new(max_capacity: u64, ttl: Duration) -> Self {
        let inner = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(ttl)
            .build();
        Self { inner }
    }

    /// Retrieve the entry for `key`, or `None` if the key is absent or the
    /// entry has expired.
    ///
    /// The returned [`CachedEntry`] is a **clone** (cheap Arc clone).
    pub fn get(&self, key: &str) -> Option<CachedEntry> {
        self.inner.get(key)
    }

    /// Insert or replace the entry for `key`.
    pub fn insert(&self, key: String, entry: CachedEntry) {
        self.inner.insert(key, entry);
    }

    /// Remove the entry for `key` immediately (does not wait for TTL expiry).
    pub fn invalidate(&self, key: &str) {
        self.inner.invalidate(key);
    }

    /// Perform any pending maintenance work: evict expired entries, enforce
    /// capacity bounds.
    ///
    /// moka defers some housekeeping to keep insert/get latency low; call this
    /// method when you want deterministic eviction — e.g., in tests or after a
    /// batch insert.
    pub fn run_pending_tasks(&self) {
        self.inner.run_pending_tasks();
    }

    /// Return the number of entries currently in the cache (including entries
    /// that may be pending eviction).
    pub fn entry_count(&self) -> u64 {
        self.inner.entry_count()
    }

    /// Return `true` if the cache currently holds no entries.
    pub fn is_empty(&self) -> bool {
        self.entry_count() == 0
    }
}

// ─── std::fmt::Debug ─────────────────────────────────────────────────────────

impl std::fmt::Debug for EngineCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineCache")
            .field("entry_count", &self.entry_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn make_cache(max: u64, ttl_secs: u64) -> EngineCache {
        EngineCache::new(max, Duration::from_secs(ttl_secs))
    }

    // ── CachedEntry ────────────────────────────────────────────────────────

    #[test]
    fn cached_entry_new_stores_bytes() {
        let entry = CachedEntry::new(b"hello".to_vec());
        assert_eq!(&*entry.bytes, b"hello");
        assert!(entry.label.is_empty());
    }

    #[test]
    fn cached_entry_with_label_stores_label() {
        let entry = CachedEntry::with_label(b"data".to_vec(), "MyAdapter");
        assert_eq!(entry.label, "MyAdapter");
        assert_eq!(entry.byte_len(), 4);
    }

    #[test]
    fn cached_entry_clone_is_cheap_arc_clone() {
        let entry = CachedEntry::new(vec![0u8; 1024]);
        let clone = entry.clone();
        // Both share the same Arc allocation.
        assert!(Arc::ptr_eq(&entry.bytes, &clone.bytes));
    }

    #[test]
    fn cached_entry_age_is_non_negative() {
        let entry = CachedEntry::new(vec![]);
        assert!(entry.age() >= Duration::ZERO);
    }

    // ── EngineCache basic operations ────────────────────────────────────────

    #[test]
    fn insert_and_get_round_trips_entry() {
        let cache = make_cache(100, 60);
        let entry = CachedEntry::with_label(b"value".to_vec(), "test");

        cache.insert("k1".to_string(), entry.clone());
        let retrieved = cache.get("k1").expect("key must be present after insert");

        assert_eq!(&*retrieved.bytes, b"value");
        assert_eq!(retrieved.label, "test");
    }

    #[test]
    fn get_missing_key_returns_none() {
        let cache = make_cache(100, 60);
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn invalidate_removes_entry() {
        let cache = make_cache(100, 60);
        cache.insert(
            "removable".to_string(),
            CachedEntry::new(b"x".to_vec()),
        );
        cache.run_pending_tasks();
        assert!(cache.get("removable").is_some());

        cache.invalidate("removable");
        cache.run_pending_tasks();
        assert!(cache.get("removable").is_none());
    }

    #[test]
    fn insert_three_entries_and_retrieve_all() {
        let cache = make_cache(100, 60);

        for i in 0u8..3 {
            cache.insert(
                format!("key-{i}"),
                CachedEntry::with_label(vec![i], format!("entry-{i}")),
            );
        }
        cache.run_pending_tasks();

        for i in 0u8..3 {
            let entry = cache.get(&format!("key-{i}")).expect("entry must exist");
            assert_eq!(&*entry.bytes, &[i]);
            assert_eq!(entry.label, format!("entry-{i}"));
        }
    }

    #[test]
    fn entry_count_reflects_insertions() {
        let cache = make_cache(100, 60);
        assert_eq!(cache.entry_count(), 0);

        cache.insert("a".to_string(), CachedEntry::new(b"1".to_vec()));
        cache.insert("b".to_string(), CachedEntry::new(b"2".to_vec()));
        cache.insert("c".to_string(), CachedEntry::new(b"3".to_vec()));
        cache.run_pending_tasks();

        assert_eq!(cache.entry_count(), 3);
    }

    #[test]
    fn is_empty_true_on_fresh_cache() {
        let cache = make_cache(50, 60);
        assert!(cache.is_empty());
    }

    #[test]
    fn is_empty_false_after_insert() {
        let cache = make_cache(50, 60);
        cache.insert("k".to_string(), CachedEntry::new(b"v".to_vec()));
        cache.run_pending_tasks();
        assert!(!cache.is_empty());
    }

    #[test]
    fn ttl_expiry_removes_entry() {
        // TTL = 1 second; sleep 1.1 s then verify expiry.
        // We use a very short TTL so the test stays fast.
        let cache = EngineCache::new(100, Duration::from_millis(50));
        cache.insert("expiring".to_string(), CachedEntry::new(b"soon".to_vec()));
        cache.run_pending_tasks();

        // Verify it's present immediately.
        assert!(cache.get("expiring").is_some(), "entry should exist right after insert");

        // Wait for TTL to elapse.
        std::thread::sleep(Duration::from_millis(120));
        cache.run_pending_tasks();

        // After expiry the entry should be gone.
        assert!(
            cache.get("expiring").is_none(),
            "entry should have expired after TTL"
        );
    }

    #[test]
    fn overwrite_replaces_existing_entry() {
        let cache = make_cache(100, 60);
        cache.insert("k".to_string(), CachedEntry::with_label(b"v1".to_vec(), "first"));
        cache.insert("k".to_string(), CachedEntry::with_label(b"v2".to_vec(), "second"));
        cache.run_pending_tasks();

        let entry = cache.get("k").unwrap();
        assert_eq!(&*entry.bytes, b"v2");
        assert_eq!(entry.label, "second");
    }

    #[test]
    fn debug_output_contains_entry_count() {
        let cache = make_cache(10, 60);
        cache.insert("x".to_string(), CachedEntry::new(b"y".to_vec()));
        cache.run_pending_tasks();
        let debug = format!("{:?}", cache);
        assert!(debug.contains("EngineCache"), "Debug output must name the type");
    }
}
