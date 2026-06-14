//! Caching wrapper adapter for engine state sources.
//!
//! [`CacheLayer`] sits in front of expensive adapter operations (e.g., target
//! directory scanning, cargo metadata queries, git status probes) and avoids
//! re-running them within the same session by storing the serialized results
//! in an [`EngineCache`].
//!
//! This is particularly useful in the adapter pipeline to prevent redundant
//! I/O: if two separate adapters both need to scan the target directory, the
//! second can retrieve the cached scan result instead of walking the filesystem
//! again.
//!
//! When the `advanced` feature is disabled, [`CacheLayer`] is a no-op stub
//! and always calls the closure to compute results.

#[cfg(feature = "advanced")]
use crate::advanced::cache::EngineCache;
#[cfg(feature = "advanced")]
use std::time::Duration;

/// A caching wrapper for adapter operations.
///
/// Wraps an [`EngineCache`] (when the `advanced` feature is enabled) to cache
/// the results of expensive closure-based computations within a single session.
#[derive(Clone)]
pub struct CacheLayer {
    #[cfg(feature = "advanced")]
    cache: Option<EngineCache>,
}

impl CacheLayer {
    /// Create a new caching layer with a given capacity (in bytes) and time-to-live (in seconds).
    ///
    /// # Arguments
    /// * `max_bytes` — The maximum total cache size, in bytes.
    /// * `ttl_secs` — The time-to-live for each entry, in seconds.
    ///
    /// When the `advanced` feature is disabled, this constructs a no-op stub
    /// that does not perform any caching.
    pub fn new(max_bytes: u64, ttl_secs: u64) -> Self {
        #[cfg(feature = "advanced")]
        {
            // Convert max_bytes to capacity (number of entries). We use a heuristic:
            // assume an average entry size and divide to get a reasonable entry count.
            // For this adapter, we assume entries are roughly 4KB on average.
            let capacity = (max_bytes / 4096).max(1);
            let ttl = Duration::from_secs(ttl_secs);
            Self {
                cache: Some(EngineCache::new(capacity, ttl)),
            }
        }

        #[cfg(not(feature = "advanced"))]
        {
            let _ = (max_bytes, ttl_secs); // silence unused warnings
            Self {}
        }
    }

    /// Retrieve a cached value by key, or compute and cache it.
    ///
    /// If the `advanced` feature is enabled and the key is cached (and not
    /// expired), returns the cached bytes directly. Otherwise, calls the
    /// closure `f`, caches the result, and returns it.
    ///
    /// # Arguments
    /// * `key` — A unique identifier for the cached entry (e.g. `"target_scan"` or `"cargo_meta"`).
    /// * `f` — A closure that computes and returns the value (as `Vec<u8>`) on a cache miss.
    ///
    /// # Returns
    /// The cached or freshly computed bytes.
    pub fn get_or_scan<F>(&self, key: &str, f: F) -> Vec<u8>
    where
        F: FnOnce() -> Vec<u8>,
    {
        #[cfg(feature = "advanced")]
        {
            if let Some(ref cache) = self.cache {
                return cache
                    .get_or_insert_with(key, f)
                    .bytes
                    .clone();
            }
            // If cache is None (shouldn't happen unless explicitly disabled), fall through.
            f()
        }

        #[cfg(not(feature = "advanced"))]
        {
            let _ = key; // silence unused warning
            f()
        }
    }
}

#[cfg(all(test, feature = "advanced"))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn cache_hit_returns_cached_value() {
        let layer = CacheLayer::new(1024 * 1024, 60);

        // First call: computes and caches.
        let result1 = layer.get_or_scan("test_key", || b"cached_data".to_vec());
        assert_eq!(result1, b"cached_data");

        // Second call: should return cached value without calling the closure.
        let result2 = layer.get_or_scan("test_key", || b"different_data".to_vec());
        assert_eq!(result2, b"cached_data", "cache hit should return the original cached value");
    }

    #[test]
    fn cache_miss_calls_closure() {
        let layer = CacheLayer::new(1024 * 1024, 60);
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        // First call with key "miss_test": closure should be called once.
        layer.get_or_scan("miss_test", || {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            b"computed".to_vec()
        });

        assert_eq!(
            call_count.load(Ordering::SeqCst),
            1,
            "closure should be called once on cache miss"
        );

        // Second call with same key: closure should not be called again.
        layer.get_or_scan("miss_test", || {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            b"recomputed".to_vec()
        });

        assert_eq!(
            call_count.load(Ordering::SeqCst),
            1,
            "closure should still be called only once due to cache hit"
        );
    }
}
