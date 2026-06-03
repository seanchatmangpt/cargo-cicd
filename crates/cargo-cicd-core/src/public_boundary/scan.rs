//! Term scanner for public boundary analysis.

use std::path::Path;

/// A forbidden-term violation found in a file.
pub struct TermViolation {
    /// File path as a string.
    pub file: String,
    /// The forbidden term found.
    pub term: String,
    /// 1-based line number.
    pub line: usize,
    /// Surrounding context (the full line).
    pub context: String,
}

/// Forbidden terms that must not appear in public-facing content.
const FORBIDDEN_TERMS: &[&str] = &[
    "ostar",
    "O*",
    "simulate",
    "simulation",
    "processing pipeline",
    "simulation pipeline",
];

/// Scan a single file for forbidden terms.
pub fn scan_file(path: &Path) -> Vec<TermViolation> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let file_str = path.to_string_lossy().into_owned();
    let mut violations = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        for term in FORBIDDEN_TERMS {
            if line.contains(term) {
                violations.push(TermViolation {
                    file: file_str.clone(),
                    term: term.to_string(),
                    line: idx + 1,
                    context: line.to_string(),
                });
            }
        }
    }

    violations
}

/// Recursively scan a directory for files with the given extensions.
pub fn scan_dir(dir: &Path, extensions: &[&str]) -> Vec<TermViolation> {
    let mut result = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return result;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            result.extend(scan_dir(&path, extensions));
        } else {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if extensions.contains(&ext) {
                result.extend(scan_file(&path));
            }
        }
    }
    result
}
