//! Concurrent, TTL-aware engine cache (backed by `moka`).
//!
//! [`EngineCache`] is a high-performance, thread-safe cache for engine
//! metadata and adapter results — for example cargo-metadata output, toolchain
//! probes, or scanned target inventories whose recomputation is expensive.
//!
//! Entries are keyed by `String` and store an [`Arc<CachedEntry>`], so reads
//! are cheap clones of a shared handle. The cache is bounded both by a maximum
//! entry capacity and by a per-entry time-to-live; eviction and expiry are
//! performed lazily by `moka` and can be forced with [`EngineCache::run_pending_tasks`].
//!
//! The cache itself is cheaply [`Clone`]able: every clone shares the same
//! underlying store, so handles can be passed freely across threads.

use std::sync::Arc;
use std::time::Duration;

use moka::sync::Cache;

/// A single cached payload plus a small piece of metadata describing it.
///
/// The `bytes` field holds the (already serialized) cached value; `label` is a
/// short, human-readable tag identifying the source or kind of the entry —
/// useful when inspecting or instrumenting the cache.
#[derive(Debug, Clone)]
pub struct CachedEntry {
    /// The cached payload bytes.
    pub bytes: Vec<u8>,
    /// A short descriptive label for the entry (e.g. the producing adapter).
    pub label: String,
}

impl CachedEntry {
    /// Construct a new entry from owned bytes with an empty label.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            label: String::new(),
        }
    }

    /// Construct a new entry with an explicit label.
    pub fn with_label(bytes: Vec<u8>, label: impl Into<String>) -> Self {
        Self {
            bytes,
            label: label.into(),
        }
    }

    /// Number of payload bytes held by this entry.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns `true` if the entry carries no payload bytes.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

/// A concurrent, bounded, TTL-aware cache for engine metadata and adapter results.
///
/// Clones share the same underlying store and are cheap to create.
#[derive(Clone)]
pub struct EngineCache {
    inner: Cache<String, Arc<CachedEntry>>,
}

impl EngineCache {
    /// Create a new cache bounded to at most `capacity` entries, where each
    /// entry expires `ttl` after it was last inserted.
    pub fn new(capacity: u64, ttl: Duration) -> Self {
        let inner = Cache::builder()
            .max_capacity(capacity)
            .time_to_live(ttl)
            .build();
        Self { inner }
    }

    /// Insert (or overwrite) the value stored under `key`.
    pub fn put(&self, key: impl Into<String>, bytes: Vec<u8>) {
        self.inner
            .insert(key.into(), Arc::new(CachedEntry::new(bytes)));
    }

    /// Insert (or overwrite) the value stored under `key`, attaching a label.
    pub fn put_labeled(&self, key: impl Into<String>, bytes: Vec<u8>, label: impl Into<String>) {
        self.inner
            .insert(key.into(), Arc::new(CachedEntry::with_label(bytes, label)));
    }

    /// Look up `key`, returning a shared handle to the entry if present and
    /// not expired.
    pub fn get(&self, key: &str) -> Option<Arc<CachedEntry>> {
        self.inner.get(key)
    }

    /// Return the entry for `key`, computing and inserting it with `f` on a miss.
    ///
    /// The closure runs at most once per missing key; concurrent callers for
    /// the same key cooperate so the value is produced a single time.
    pub fn get_or_insert_with(
        &self,
        key: impl Into<String>,
        f: impl FnOnce() -> Vec<u8>,
    ) -> Arc<CachedEntry> {
        self.inner
            .get_with(key.into(), || Arc::new(CachedEntry::new(f())))
    }

    /// Remove the entry stored under `key`, if any.
    pub fn invalidate(&self, key: &str) {
        self.inner.invalidate(key);
    }

    /// The current number of entries, after any already-applied maintenance.
    ///
    /// Pending eviction/expiry may not yet be reflected; call
    /// [`EngineCache::run_pending_tasks`] first for an exact figure.
    pub fn entry_count(&self) -> u64 {
        self.inner.entry_count()
    }

    /// Force any pending maintenance (eviction, expiry) to settle.
    ///
    /// Useful in tests that need a deterministic view of the cache state.
    pub fn run_pending_tasks(&self) {
        self.inner.run_pending_tasks();
    }
}

impl std::fmt::Debug for EngineCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineCache")
            .field("entry_count", &self.inner.entry_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread::sleep;

    fn cache() -> EngineCache {
        EngineCache::new(128, Duration::from_secs(60))
    }

    #[test]
    fn put_get_roundtrip() {
        let c = cache();
        c.put("alpha", b"hello".to_vec());
        let entry = c.get("alpha").expect("entry should be present");
        assert_eq!(entry.bytes, b"hello");
        assert!(!entry.is_empty());
        assert_eq!(entry.len(), 5);
    }

    #[test]
    fn miss_returns_none() {
        let c = cache();
        assert!(c.get("absent").is_none());
    }

    #[test]
    fn capacity_based_eviction() {
        let cap: u64 = 8;
        let c = EngineCache::new(cap, Duration::from_secs(60));
        for i in 0..(cap * 4) {
            c.put(format!("key-{i}"), vec![i as u8]);
        }
        c.run_pending_tasks();
        assert!(
            c.entry_count() <= cap,
            "entry_count {} should be bounded by capacity {}",
            c.entry_count(),
            cap
        );
    }

    #[test]
    fn ttl_expiry() {
        let c = EngineCache::new(128, Duration::from_millis(20));
        c.put("ephemeral", b"bye".to_vec());
        c.run_pending_tasks();
        assert!(c.get("ephemeral").is_some());

        sleep(Duration::from_millis(60));
        c.run_pending_tasks();
        assert!(
            c.get("ephemeral").is_none(),
            "entry should have expired after its TTL"
        );
    }

    #[test]
    fn get_or_insert_with_runs_closure_only_on_miss() {
        let c = cache();
        let calls = AtomicUsize::new(0);

        let first = c.get_or_insert_with("lazy", || {
            calls.fetch_add(1, Ordering::SeqCst);
            b"computed".to_vec()
        });
        assert_eq!(first.bytes, b"computed");

        let second = c.get_or_insert_with("lazy", || {
            calls.fetch_add(1, Ordering::SeqCst);
            b"recomputed".to_vec()
        });
        assert_eq!(second.bytes, b"computed", "second call must reuse cached value");
        assert_eq!(calls.load(Ordering::SeqCst), 1, "closure must run only once");
    }

    #[test]
    fn invalidate_removes_entry() {
        let c = cache();
        c.put("temp", b"data".to_vec());
        assert!(c.get("temp").is_some());
        c.invalidate("temp");
        c.run_pending_tasks();
        assert!(c.get("temp").is_none());
    }

    #[test]
    fn cache_is_cloneable_and_shares_store() {
        let c = cache();
        let clone = c.clone();
        c.put("shared", b"v".to_vec());
        assert!(clone.get("shared").is_some());
    }
}
