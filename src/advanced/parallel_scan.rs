//! Hyper-fast, gitignore-aware parallel workspace scanner.
//!
//! This module pairs [`ignore::WalkBuilder`] (a multi-threaded, gitignore-aware
//! directory walker) with [`rayon`] data-parallel aggregation to produce a
//! [`ScanReport`] for a workspace root. The scan answers the questions a CI/CD
//! helper cares about: how many files are present, how big the tree is, how the
//! bytes break down per file extension, and how many bytes sit under `target/`
//! directories (i.e. reclaimable build output).
//!
//! The walk itself runs across all available cores via the underlying walker's
//! parallel mode; file sizing is then aggregated with rayon so large trees are
//! processed without serial bottlenecks.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

use ignore::{WalkBuilder, WalkState};
use rayon::prelude::*;

/// Per-extension rollup of file counts and byte totals.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExtensionStats {
    /// Number of files observed with this extension.
    pub count: u64,
    /// Total number of bytes across files with this extension.
    pub bytes: u64,
}

/// Aggregated result of scanning a workspace tree.
///
/// All maps are backed by [`BTreeMap`] so iteration order is deterministic,
/// which keeps downstream receipts and test assertions stable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanReport {
    /// Total number of regular files discovered.
    pub total_files: u64,
    /// Total number of bytes across all discovered files.
    pub total_bytes: u64,
    /// Per-extension counts and byte totals, keyed by lowercased extension.
    ///
    /// Files without an extension are grouped under the key `"<none>"`.
    pub per_extension: BTreeMap<String, ExtensionStats>,
    /// Discovered `target/` directories mapped to their reclaimable byte totals.
    pub target_dirs: BTreeMap<PathBuf, u64>,
}

impl ScanReport {
    /// Total bytes that could be reclaimed by removing every discovered
    /// `target/` directory.
    pub fn reclaimable_bytes(&self) -> u64 {
        self.target_dirs.values().copied().sum()
    }

    /// Number of distinct file extensions observed (the `"<none>"` bucket
    /// counts as one if any extension-less files were seen).
    pub fn distinct_extensions(&self) -> usize {
        self.per_extension.len()
    }

    /// The extension accounting for the most bytes, paired with its stats.
    ///
    /// Returns [`None`] when no files were scanned. Ties are broken by
    /// extension name to keep the result deterministic.
    pub fn largest_extension(&self) -> Option<(&str, ExtensionStats)> {
        self.per_extension
            .iter()
            .max_by(|a, b| a.1.bytes.cmp(&b.1.bytes).then_with(|| b.0.cmp(a.0)))
            .map(|(ext, stats)| (ext.as_str(), *stats))
    }

    /// Stats for a single extension, if present. Pass `"<none>"` for the
    /// extension-less bucket.
    pub fn extension(&self, ext: &str) -> Option<ExtensionStats> {
        self.per_extension.get(ext).copied()
    }
}

/// Key used for files that have no file-name extension.
const NO_EXTENSION_KEY: &str = "<none>";

/// Directory name that marks reclaimable build output.
const TARGET_DIR_NAME: &str = "target";

/// Lowercased extension key for a path, or [`NO_EXTENSION_KEY`].
fn extension_key(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_else(|| NO_EXTENSION_KEY.to_string())
}

/// Whether any component of `path` is a `target` directory.
fn is_under_target(path: &Path) -> bool {
    path.components().any(|c| match c {
        Component::Normal(name) => name == TARGET_DIR_NAME,
        _ => false,
    })
}

/// The deepest enclosing `target/` directory for `path`, relative to the walk.
///
/// Given `.../target/debug/foo.rlib` this returns `.../target`. Returns [`None`]
/// when the path is not under a `target` directory.
fn enclosing_target_dir(path: &Path) -> Option<PathBuf> {
    let mut acc = PathBuf::new();
    for component in path.components() {
        acc.push(component.as_os_str());
        if let Component::Normal(name) = component {
            if name == TARGET_DIR_NAME {
                return Some(acc);
            }
        }
    }
    None
}

/// Construct a parallel, gitignore-aware walker rooted at `root`.
///
/// The walker honours `.gitignore`/`.ignore` rules and hidden-file conventions
/// (consistent with how a developer's working tree is interpreted) and is set to
/// fan out across all available cores. `target/` directories are intentionally
/// *not* excluded here: callers depend on seeing reclaimable build output.
fn build_walker(root: &Path) -> ignore::WalkParallel {
    WalkBuilder::new(root)
        .standard_filters(true)
        .threads(num_threads())
        .build_parallel()
}

/// Number of worker threads for the parallel walk.
///
/// Uses the available parallelism reported by the runtime, falling back to a
/// single thread when that figure cannot be determined.
fn num_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// Scan a workspace tree rooted at `root`, returning an aggregated report.
///
/// The directory walk is gitignore-aware and runs in parallel across the
/// available cores. File sizes are then aggregated in parallel with rayon. Only
/// regular files contribute to counts and byte totals; directories and other
/// special entries are skipped.
///
/// # Errors
///
/// Returns an [`std::io::Error`] if the root cannot be walked. Individual
/// entries that fail to read (e.g. due to permissions) are skipped rather than
/// aborting the whole scan.
pub fn scan(root: &Path) -> std::io::Result<ScanReport> {
    // Phase 1: parallel, gitignore-aware walk that collects (path, size) pairs
    // for every regular file. A Mutex-guarded Vec is the thread-safe sink; the
    // walk threads do only cheap metadata reads, so contention stays low.
    let collected: Mutex<Vec<(PathBuf, u64)>> = Mutex::new(Vec::new());

    let walker = build_walker(root);

    walker.run(|| {
        let collected = &collected;
        Box::new(move |result| {
            if let Ok(entry) = result {
                let is_file = entry.file_type().map(|ft| ft.is_file()).unwrap_or(false);
                if is_file {
                    if let Ok(meta) = entry.metadata() {
                        let path = entry.path().to_path_buf();
                        collected.lock().unwrap().push((path, meta.len()));
                    }
                }
            }
            WalkState::Continue
        })
    });

    let files = collected.into_inner().unwrap();

    // Phase 2: parallel aggregation with rayon. Each file folds into a partial
    // report; partials are reduced pairwise into the final report.
    let report = files
        .par_iter()
        .fold(ScanReport::default, |mut acc, (path, size)| {
            acc.total_files += 1;
            acc.total_bytes += size;

            let ext = extension_key(path);
            let stats = acc.per_extension.entry(ext).or_default();
            stats.count += 1;
            stats.bytes += size;

            if let Some(target_dir) = enclosing_target_dir(path) {
                *acc.target_dirs.entry(target_dir).or_insert(0) += size;
            }
            acc
        })
        .reduce(ScanReport::default, merge_reports);

    Ok(report)
}

/// Merge two partial reports into one. Associative + commutative so it is safe
/// as a rayon reduction combiner.
fn merge_reports(mut a: ScanReport, b: ScanReport) -> ScanReport {
    a.total_files += b.total_files;
    a.total_bytes += b.total_bytes;

    for (ext, stats) in b.per_extension {
        let entry = a.per_extension.entry(ext).or_default();
        entry.count += stats.count;
        entry.bytes += stats.bytes;
    }

    for (dir, bytes) in b.target_dirs {
        *a.target_dirs.entry(dir).or_insert(0) += bytes;
    }

    a
}

/// Total bytes reclaimable by clearing every `target/` directory under `root`.
///
/// This is a focused convenience wrapper: it performs a parallel walk and sums
/// the sizes of all files that live under any `target/` directory. Prefer
/// [`scan`] when you also need per-extension or total figures.
///
/// On any walk error the partial total computed so far is returned, since a
/// best-effort reclaimable estimate is more useful to callers than a hard
/// failure.
pub fn reclaimable_target_bytes(root: &Path) -> u64 {
    reclaimable_target_bytes_with_errors(root)
        .map(|(bytes, _)| bytes)
        .unwrap_or(0)
}

/// Like [`reclaimable_target_bytes`], but also reports how many entries
/// could not be read (permission errors, races, etc.) during the walk or
/// per-file metadata reads, so callers can surface an undercount signal
/// instead of silently swallowing it.
pub fn reclaimable_target_bytes_with_errors(root: &Path) -> std::io::Result<(u64, usize)> {
    let collected: Mutex<Vec<(PathBuf, u64)>> = Mutex::new(Vec::new());
    let errors = std::sync::atomic::AtomicUsize::new(0);

    let walker = build_walker(root);

    walker.run(|| {
        let collected = &collected;
        let errors = &errors;
        Box::new(move |result| {
            match result {
                Ok(entry) => {
                    let is_file = entry.file_type().map(|ft| ft.is_file()).unwrap_or(false);
                    if is_file && is_under_target(entry.path()) {
                        match entry.metadata() {
                            Ok(meta) => {
                                collected
                                    .lock()
                                    .unwrap()
                                    .push((entry.path().to_path_buf(), meta.len()));
                            }
                            Err(_) => {
                                errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    }
                }
                Err(_) => {
                    errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
            WalkState::Continue
        })
    });

    let total: u64 = collected
        .into_inner()
        .unwrap()
        .par_iter()
        .map(|(_, size)| *size)
        .sum();

    Ok((total, errors.into_inner()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    /// Write `contents` to `path`, creating parent directories as needed.
    fn write_file(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(path).unwrap();
        f.write_all(contents).unwrap();
    }

    /// Build a small workspace tree with source files and a fake `target/` dir.
    fn build_fixture(root: &Path) {
        write_file(&root.join("src/main.rs"), b"fn main() {}\n"); // 13 bytes
        write_file(&root.join("src/lib.rs"), b"pub fn x() {}\n"); // 14 bytes
        write_file(&root.join("README.md"), b"# title\n"); // 8 bytes
        write_file(&root.join("Cargo.toml"), b"[package]\n"); // 10 bytes
                                                              // Fake build output under target/.
        write_file(&root.join("target/debug/app"), b"BINARYDATA"); // 10 bytes
        write_file(&root.join("target/debug/app.d"), b"deps:\n"); // 6 bytes
    }

    #[test]
    fn counts_and_byte_totals_are_accurate() {
        let dir = tempfile::tempdir().unwrap();
        build_fixture(dir.path());

        let report = scan(dir.path()).unwrap();

        // 4 source/config files + 2 target files = 6 files total.
        assert_eq!(report.total_files, 6);
        // 13 + 14 + 8 + 10 + 10 + 6 = 61 bytes.
        assert_eq!(report.total_bytes, 61);
    }

    #[test]
    fn per_extension_rollup_is_correct() {
        let dir = tempfile::tempdir().unwrap();
        build_fixture(dir.path());

        let report = scan(dir.path()).unwrap();

        let rs = report.extension("rs").expect("rs bucket present");
        assert_eq!(rs.count, 2);
        assert_eq!(rs.bytes, 27); // main.rs (13) + lib.rs (14)

        let md = report.extension("md").expect("md bucket present");
        assert_eq!(md.count, 1);
        assert_eq!(md.bytes, 8);

        // The `app` binary has no extension and lands in the <none> bucket.
        let none = report.extension(NO_EXTENSION_KEY).expect("none bucket");
        assert_eq!(none.count, 1);
        assert_eq!(none.bytes, 10);

        // `rs` is the largest extension by bytes (27).
        let (ext, stats) = report.largest_extension().expect("non-empty report");
        assert_eq!(ext, "rs");
        assert_eq!(stats.bytes, 27);
    }

    #[test]
    fn reclaimable_target_bytes_sums_target_tree() {
        let dir = tempfile::tempdir().unwrap();
        build_fixture(dir.path());

        // app (10) + app.d (6) = 16 reclaimable bytes.
        assert_eq!(reclaimable_target_bytes(dir.path()), 16);

        let report = scan(dir.path()).unwrap();
        assert_eq!(report.reclaimable_bytes(), 16);

        // Exactly one target directory was discovered.
        assert_eq!(report.target_dirs.len(), 1);
        let (_, &bytes) = report.target_dirs.iter().next().unwrap();
        assert_eq!(bytes, 16);
    }

    #[test]
    fn empty_tree_yields_empty_report() {
        let dir = tempfile::tempdir().unwrap();
        let report = scan(dir.path()).unwrap();

        assert_eq!(report.total_files, 0);
        assert_eq!(report.total_bytes, 0);
        assert_eq!(report.distinct_extensions(), 0);
        assert!(report.largest_extension().is_none());
        assert_eq!(report.reclaimable_bytes(), 0);
    }
}
