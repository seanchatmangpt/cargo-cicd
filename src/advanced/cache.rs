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
// Public API surface exercised by `examples/03_max_pipeline.rs` (tutorial
// anchor for docs/tutorials/03-full-pipeline.md), which is compiled as a
// separate cargo target and so doesn't suppress `cargo build`'s dead_code
// lint on the library crate.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CachedEntry {
    /// The cached payload bytes.
    pub bytes: Vec<u8>,
    /// A short descriptive label for the entry (e.g. the producing adapter).
    pub label: String,
}

#[allow(dead_code)] // exercised by examples/03_max_pipeline.rs
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
#[allow(dead_code)] // exercised by examples/03_max_pipeline.rs
#[derive(Clone)]
pub struct EngineCache {
    inner: Cache<String, Arc<CachedEntry>>,
}

#[allow(dead_code)] // exercised by examples/03_max_pipeline.rs
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
