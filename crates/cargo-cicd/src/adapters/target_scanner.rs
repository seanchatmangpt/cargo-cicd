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

impl Default for TargetScannerAdapter {
    fn default() -> Self {
        Self::new()
    }
}
