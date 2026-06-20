//! Advanced capabilities — opt-in via `--features advanced`.
//!
//! All modules are feature-gated. The default binary stays lean.
//!
//! | Module | Crate | Use Case |
//! |--------|-------|----------|
//! | `parallel_scan` | `ignore` + `rayon` | Gitignore-aware multi-threaded workspace scanning |
//! | `fingerprint`   | `blake3`            | BLAKE3 content-addressed artifact hashing |
//! | `observability` | `tracing`           | Structured span instrumentation + JSON traces |
//! | `cache`         | `moka`              | Concurrent TTL-aware result caching |
//!
//! # Quick Start
//!
//! ```rust,ignore
//! // Scan a workspace (parallel, gitignore-aware)
//! let report = parallel_scan::scan_workspace(Path::new("."))?;
//!
//! // Fingerprint a file
//! let manifest = fingerprint::fingerprint_file(Path::new("Cargo.lock"))?;
//!
//! // Instrument a pipeline stage
//! let result = observability::with_stage("my_adapter", || my_work());
//!
//! // Cache adapter results
//! let cache = cache::EngineCache::new(100, Duration::from_secs(300));
//! cache.insert("key".to_string(), cache::CachedEntry::with_label(bytes, "label"));
//! ```

pub mod cache;
pub mod fingerprint;
pub mod observability;
pub mod parallel_scan;
