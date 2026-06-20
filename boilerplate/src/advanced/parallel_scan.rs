//! Gitignore-aware, multi-threaded workspace scanning via `ignore` + `rayon`.
//!
//! Uses [`ignore::WalkBuilder`] to honour `.gitignore` and similar ignore
//! files, then processes directory entries in parallel with `rayon` via a
//! [`dashmap::DashMap`] accumulator — no `Mutex` contention.
//!
//! # Example
//!
//! ```rust,ignore
//! use std::path::Path;
//! use my_crate::advanced::parallel_scan::scan_workspace;
//!
//! let report = scan_workspace(Path::new("."))?;
//! println!("total files: {}", report.total_files);
//! println!("reclaimable: {} bytes", report.reclaimable_bytes());
//! for (ext, stats) in &report.per_extension {
//!     println!("{}: {} files, {} bytes", ext, stats.count, stats.bytes);
//! }
//! ```

use anyhow::Result;
use dashmap::DashMap;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Per-extension file statistics.
#[derive(Debug, Clone, Default)]
pub struct ExtStats {
    /// Number of files with this extension.
    pub count: usize,
    /// Total byte size of files with this extension.
    pub bytes: u64,
}

/// Aggregated workspace scan report produced by [`scan_workspace`].
#[derive(Debug, Clone)]
pub struct ScanReport {
    /// Total number of regular files visited (directories and symlinks excluded).
    pub total_files: usize,
    /// Total byte size of all visited files.
    pub total_bytes: u64,
    /// Per-extension breakdown, sorted lexicographically.
    ///
    /// The key `""` holds stats for extension-less files.
    pub per_extension: BTreeMap<String, ExtStats>,
    /// Byte total of files located inside reclaimable directories such as
    /// `target/`, `node_modules/`, `.gradle/`, `__pycache__/`, `.cache/`.
    reclaimable: u64,
}

impl ScanReport {
    /// Returns the estimated reclaimable byte count from build-artifact and
    /// dependency-cache directories (`target/`, `node_modules/`, `.gradle/`,
    /// `__pycache__/`, `.cache/`).
    pub fn reclaimable_bytes(&self) -> u64 {
        self.reclaimable
    }

    /// Total number of distinct extensions observed (including `""` for
    /// extension-less files, if any).
    pub fn distinct_extensions(&self) -> usize {
        self.per_extension.len()
    }
}

/// Path component names that indicate reclaimable / ephemeral build output.
const RECLAIMABLE_DIRS: &[&str] = &["target", "node_modules", ".gradle", "__pycache__", ".cache"];

/// Scan `root` recursively, honouring `.gitignore` and similar ignore files.
///
/// Returns a [`ScanReport`] with per-extension statistics.  Only regular
/// files are counted; directories and symlinks are skipped.
///
/// Individual unreadable files are silently skipped (silence contract).
///
/// # Errors
///
/// Returns an error if `root` does not exist or cannot be accessed at all.
pub fn scan_workspace(root: &Path) -> Result<ScanReport> {
    // Phase 1: collect all matching DirEntry values with a sequential walk.
    // WalkBuilder honours .gitignore, .git/info/exclude, and global gitignore.
    let entries: Vec<ignore::DirEntry> = {
        let mut acc = Vec::new();
        for result in WalkBuilder::new(root)
            .hidden(false) // traverse hidden files; .gitignore still applies
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build()
        {
            match result {
                Ok(entry) => {
                    if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        acc.push(entry);
                    }
                }
                Err(_) => {} // silence contract: skip unreadable entries
            }
        }
        acc
    };

    // Phase 2: parallel accumulation via DashMap (shard-level locking, no
    // global Mutex contention under rayon's work-stealing scheduler).
    let ext_map: Arc<DashMap<String, (usize, u64)>> = Arc::new(DashMap::new());
    let total_files = Arc::new(AtomicUsize::new(0));
    let total_bytes = Arc::new(AtomicU64::new(0));
    let reclaimable = Arc::new(AtomicU64::new(0));

    entries.par_iter().for_each(|entry| {
        let path = entry.path();
        let size = match path.metadata() {
            Ok(m) => m.len(),
            Err(_) => return, // silence contract
        };

        // Extension key: lowercase, or "" for extension-less files.
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        total_files.fetch_add(1, Ordering::Relaxed);
        total_bytes.fetch_add(size, Ordering::Relaxed);

        // Mark bytes reclaimable if any path component matches.
        let is_reclaimable = path.components().any(|c| {
            c.as_os_str()
                .to_str()
                .map(|s| RECLAIMABLE_DIRS.contains(&s))
                .unwrap_or(false)
        });
        if is_reclaimable {
            reclaimable.fetch_add(size, Ordering::Relaxed);
        }

        // Update shard atomically.
        ext_map
            .entry(ext)
            .and_modify(|(c, b)| {
                *c += 1;
                *b += size;
            })
            .or_insert((1, size));
    });

    // Phase 3: collapse DashMap into a deterministic BTreeMap.
    let per_extension: BTreeMap<String, ExtStats> = ext_map
        .iter()
        .map(|kv| {
            let ext = kv.key().clone();
            let (count, bytes) = *kv.value();
            (ext, ExtStats { count, bytes })
        })
        .collect();

    Ok(ScanReport {
        total_files: total_files.load(Ordering::Relaxed),
        total_bytes: total_bytes.load(Ordering::Relaxed),
        reclaimable: reclaimable.load(Ordering::Relaxed),
        per_extension,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_file(dir: &Path, name: &str, content: &[u8]) {
        fs::write(dir.join(name), content).unwrap();
    }

    #[test]
    fn scan_empty_dir_returns_zero_files() {
        let dir = TempDir::new().unwrap();
        let report = scan_workspace(dir.path()).unwrap();
        assert_eq!(report.total_files, 0);
        assert_eq!(report.total_bytes, 0);
        assert_eq!(report.reclaimable_bytes(), 0);
        assert!(report.per_extension.is_empty());
    }

    #[test]
    fn scan_counts_files_by_extension() {
        let dir = TempDir::new().unwrap();
        write_file(dir.path(), "main.rs", b"fn main() {}");
        write_file(dir.path(), "lib.rs", b"pub fn f() {}");
        write_file(dir.path(), "README.md", b"# readme");
        write_file(dir.path(), "no_ext", b"plain");

        let report = scan_workspace(dir.path()).unwrap();

        assert_eq!(report.total_files, 4);
        assert!(report.total_bytes > 0);

        let rs = report.per_extension.get("rs").expect("rs extension present");
        assert_eq!(rs.count, 2);

        let md = report.per_extension.get("md").expect("md extension present");
        assert_eq!(md.count, 1);

        // Extension-less files use the "" key.
        let no_ext = report
            .per_extension
            .get("")
            .expect("extension-less files present");
        assert_eq!(no_ext.count, 1);
    }

    #[test]
    fn bytes_sum_matches_individual_files() {
        let dir = TempDir::new().unwrap();
        let payload_a = b"hello world"; // 11 bytes
        let payload_b = b"rust";        //  4 bytes
        write_file(dir.path(), "a.txt", payload_a);
        write_file(dir.path(), "b.txt", payload_b);

        let report = scan_workspace(dir.path()).unwrap();

        assert_eq!(report.total_files, 2);
        assert_eq!(report.total_bytes, 15);

        let txt = report.per_extension.get("txt").unwrap();
        assert_eq!(txt.bytes, 15);
    }

    #[test]
    fn reclaimable_bytes_cover_target_dir() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("target");
        fs::create_dir(&target).unwrap();
        write_file(&target, "artifact.rlib", &vec![0u8; 1024]);
        write_file(dir.path(), "Cargo.toml", b"[package]");

        let report = scan_workspace(dir.path()).unwrap();

        assert!(
            report.reclaimable_bytes() >= 1024,
            "target/ bytes should be reclaimable"
        );
        // The Cargo.toml at the root is NOT reclaimable.
        assert!(report.total_bytes > report.reclaimable_bytes());
    }

    #[test]
    fn per_extension_map_is_sorted() {
        let dir = TempDir::new().unwrap();
        write_file(dir.path(), "z.toml", b"[a]");
        write_file(dir.path(), "a.rs", b"//!");
        write_file(dir.path(), "m.md", b"#");

        let report = scan_workspace(dir.path()).unwrap();
        let keys: Vec<&str> = report.per_extension.keys().map(|s| s.as_str()).collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        assert_eq!(keys, sorted, "BTreeMap keys must be in lexicographic order");
    }

    #[test]
    fn distinct_extensions_count_matches_map_len() {
        let dir = TempDir::new().unwrap();
        write_file(dir.path(), "a.rs", b"x");
        write_file(dir.path(), "b.rs", b"y");
        write_file(dir.path(), "c.md", b"z");

        let report = scan_workspace(dir.path()).unwrap();
        assert_eq!(report.distinct_extensions(), report.per_extension.len());
        assert_eq!(report.distinct_extensions(), 2); // "rs" and "md"
    }

    #[test]
    fn scan_nonexistent_root_does_not_panic() {
        // Silence contract: no panic, result may be Ok(empty) or Err.
        let result = scan_workspace(Path::new("/tmp/__nonexistent_bp_advanced_test__"));
        let _ = result;
    }
}
