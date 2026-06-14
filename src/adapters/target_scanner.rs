use std::path::Path;
use walkdir::WalkDir;

pub struct TargetScannerAdapter;

impl TargetScannerAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn total_size_bytes(target_dir: &str) -> u64 {
        let path = Path::new(target_dir);
        if !path.exists() {
            return 0;
        }
        WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum()
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
    pub fn parallel_scan_if_available(&self, root: &Path) -> Option<crate::advanced::parallel_scan::ScanReport> {
        crate::advanced::parallel_scan::scan(root).ok()
    }
}

#[cfg(not(feature = "advanced"))]
impl TargetScannerAdapter {
    /// Fallback stub for when `advanced` feature is disabled.
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
