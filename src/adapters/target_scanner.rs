use std::path::Path;
use walkdir::WalkDir;

#[cfg(feature = "advanced")]
use super::super::advanced::parallel_scan;

pub struct TargetScannerAdapter;

impl TargetScannerAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Total size in bytes of `target_dir`, along with a count of entries
    /// that could not be read (permission errors, races, etc). A non-zero
    /// error count means the returned size may be an undercount.
    pub fn total_size_bytes_with_errors(target_dir: &str) -> (u64, usize) {
        let path = Path::new(target_dir);
        if !path.exists() {
            return (0, 0);
        }

        #[cfg(feature = "advanced")]
        {
            Self::scan_fast(path)
        }
        #[cfg(not(feature = "advanced"))]
        {
            Self::scan_sequential(path)
        }
    }

    pub fn total_size_bytes(target_dir: &str) -> u64 {
        Self::total_size_bytes_with_errors(target_dir).0
    }

    fn scan_sequential(path: &Path) -> (u64, usize) {
        let mut total = 0u64;
        let mut errors = 0usize;
        for entry in WalkDir::new(path).into_iter() {
            match entry {
                Ok(e) => {
                    if e.file_type().is_file() {
                        match e.metadata() {
                            Ok(m) => total += m.len(),
                            Err(_) => errors += 1,
                        }
                    }
                }
                Err(_) => errors += 1,
            }
        }
        (total, errors)
    }

    #[cfg(feature = "advanced")]
    fn scan_fast(path: &Path) -> (u64, usize) {
        match parallel_scan::reclaimable_target_bytes_with_errors(path) {
            Ok((bytes, errors)) => (bytes, errors),
            Err(_) => Self::scan_sequential(path),
        }
    }

    pub fn total_size_gb(target_dir: &str) -> f64 {
        Self::total_size_bytes(target_dir) as f64 / 1_073_741_824.0
    }

    pub fn verdict(size_gb: f64, max_gb: f64) -> &'static str {
        if size_gb < max_gb * 0.7 {
            "pass"
        } else if size_gb < max_gb {
            "warn"
        } else {
            "fail"
        }
    }
}

#[cfg(feature = "advanced")]
impl TargetScannerAdapter {
    /// Use the parallel_scan module to scan a workspace root and return
    /// detailed scan report. Only available when the `advanced` feature is enabled.
    pub fn parallel_scan_if_available(&self, root: &Path) -> Option<parallel_scan::ScanReport> {
        parallel_scan::scan(root).ok()
    }
}

#[cfg(not(feature = "advanced"))]
impl TargetScannerAdapter {
    /// Fallback stub for when `advanced` feature is disabled. Kept for API
    /// symmetry with the `advanced`-gated impl above; only called by callers
    /// built with `advanced` off, none of which currently exist in this tree,
    /// so it warns dead here even though it is load-bearing API surface.
    #[allow(dead_code)]
    pub fn parallel_scan_if_available(&self, _root: &Path) -> Option<()> {
        None
    }
}

impl Default for TargetScannerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(all(test, feature = "advanced"))]
    mod advanced_tests {
        use super::*;
        use std::fs;
        use std::io::Write;

        fn write_file(path: &Path, contents: &[u8]) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            let mut f = fs::File::create(path).unwrap();
            f.write_all(contents).unwrap();
        }

        #[test]
        fn parallel_scan_if_available_returns_some_when_enabled() {
            let dir = tempfile::tempdir().unwrap();
            write_file(&dir.path().join("src/main.rs"), b"fn main() {}\n");
            write_file(&dir.path().join("target/debug/app"), b"BINARYDATA");

            let adapter = TargetScannerAdapter::new();
            let report = adapter.parallel_scan_if_available(dir.path());

            assert!(report.is_some());
            let report = report.unwrap();
            assert!(report.total_files > 0);
            assert!(report.reclaimable_bytes() > 0);
        }

        #[test]
        fn parallel_scan_captures_target_directory_bytes() {
            let dir = tempfile::tempdir().unwrap();
            write_file(&dir.path().join("README.md"), b"# test\n");
            write_file(&dir.path().join("target/debug/lib.rlib"), b"LIBDATA123");

            let adapter = TargetScannerAdapter::new();
            let report = adapter
                .parallel_scan_if_available(dir.path())
                .expect("report should be available");

            // Verify target/ bytes are accounted for.
            assert_eq!(report.reclaimable_bytes(), 10);
        }
    }
}
