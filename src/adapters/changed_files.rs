use std::path::{Path, PathBuf};

/// Partition a slice of file paths into source files and test files.
///
/// Returns `(source_files, test_files)`.  A file is classified as a test file if it
/// lives under a `tests/` directory, or its name ends with `_test.rs` / `_tests.rs`.
/// All other `.rs` files are source files.
pub fn classify_rust_files(files: &[PathBuf]) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut source = Vec::new();
    let mut tests = Vec::new();
    for f in files {
        if is_test_file(f) {
            tests.push(f.clone());
        } else if f.extension().and_then(|e| e.to_str()) == Some("rs") {
            source.push(f.clone());
        }
    }
    (source, tests)
}

fn is_test_file(path: &Path) -> bool {
    let lossy = path.to_string_lossy();
    if lossy.contains("/tests/") || lossy.contains("\\tests\\") {
        return true;
    }
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        return name.ends_with("_test.rs") || name.ends_with("_tests.rs");
    }
    false
}

/// Derive test name hints from a list of changed file paths.
///
/// Heuristics:
/// - `src/foo.rs`         → `test_foo`
/// - `src/foo/bar.rs`     → `test_bar`
/// - `tests/foo_test.rs`  → `foo_test` (stem as-is)
/// - `tests/foo.rs`       → `foo`
pub fn derive_test_names(changed: &[PathBuf]) -> Vec<String> {
    let mut names = Vec::new();
    for path in changed {
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };
        let lossy = path.to_string_lossy();
        // Files under tests/ — use the stem directly.
        if lossy.contains("/tests/") || lossy.contains("\\tests\\") {
            names.push(stem.to_string());
        } else {
            // Source file — prefix with test_.
            names.push(format!("test_{}", stem));
        }
    }
    names
}
