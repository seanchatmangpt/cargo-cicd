use walkdir::WalkDir;

pub struct TrybuildDetector;

impl TrybuildDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn all_fixtures(workspace_root: &str) -> Vec<String> {
        WalkDir::new(workspace_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                let path = e.path().to_string_lossy();
                (path.contains("/tests/") || path.contains("\\tests\\"))
                    && (path.ends_with(".rs")
                        || path.ends_with(".stderr")
                        || path.ends_with(".stdout"))
            })
            .map(|e| e.path().to_string_lossy().to_string())
            .collect()
    }
}

impl Default for TrybuildDetector {
    fn default() -> Self {
        Self::new()
    }
}
