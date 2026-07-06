//! Minimal, dependency-free glob matching for standing ingestors.
//!
//! Supports `*` (matches within one path segment) and `**` (matches zero or
//! more path segments) — enough for patterns like `target/**/plan.json` or
//! `benches/*.txt` without pulling in the `glob` crate. No new dependency:
//! this crate already carries `walkdir` for directory traversal, reused here.

use std::path::{Path, PathBuf};

/// Expand a glob pattern to the set of existing files that match it.
///
/// If the pattern contains no `*`, it is treated as a literal path: the
/// vector contains that single path if it exists, and is empty otherwise
/// (never panics, never fabricates a match).
pub fn expand(pattern: &str) -> Vec<PathBuf> {
    if !pattern.contains('*') {
        let p = PathBuf::from(pattern);
        return if p.exists() { vec![p] } else { vec![] };
    }

    let normalized = pattern.replace('\\', "/");
    let base = literal_prefix_dir(&normalized);
    let base_dir = if base.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        base
    };
    if !base_dir.is_dir() {
        return vec![];
    }

    let mut matches = vec![];
    for entry in walkdir::WalkDir::new(&base_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let path_str = path.to_string_lossy().replace('\\', "/");
        if glob_match(&normalized, &path_str) {
            matches.push(path.to_path_buf());
        }
    }
    matches.sort();
    matches
}

/// The longest directory prefix of `pattern` containing no wildcard segment.
fn literal_prefix_dir(pattern: &str) -> PathBuf {
    let mut out: Vec<&str> = vec![];
    for seg in pattern.split('/') {
        if seg.contains('*') {
            break;
        }
        out.push(seg);
    }
    PathBuf::from(out.join("/"))
}

/// Match `text` (a `/`-separated path) against `pattern`, where `*` matches
/// any run of characters within a single segment and `**` matches zero or
/// more whole segments.
pub fn glob_match(pattern: &str, text: &str) -> bool {
    let pat_segs: Vec<&str> = pattern.split('/').collect();
    let text_segs: Vec<&str> = text.split('/').collect();
    match_segs(&pat_segs, &text_segs)
}

fn match_segs(pat: &[&str], text: &[&str]) -> bool {
    match pat.first() {
        None => text.is_empty(),
        Some(&"**") => {
            if pat.len() == 1 {
                return true;
            }
            // Try consuming 0..=text.len() segments for "**".
            for i in 0..=text.len() {
                if match_segs(&pat[1..], &text[i..]) {
                    return true;
                }
            }
            false
        }
        Some(seg) => {
            if text.is_empty() {
                return false;
            }
            segment_match(seg, text[0]) && match_segs(&pat[1..], &text[1..])
        }
    }
}

/// Match a single path segment against a pattern segment containing `*`
/// wildcards (glob-star within the segment only, no path separators).
fn segment_match(pattern: &str, text: &str) -> bool {
    let pat_bytes: Vec<&str> = pattern.split('*').collect();
    if pat_bytes.len() == 1 {
        return pattern == text;
    }
    let mut rest = text;
    for (i, part) in pat_bytes.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !rest.starts_with(part) {
                return false;
            }
            rest = &rest[part.len()..];
        } else if i == pat_bytes.len() - 1 {
            if !rest.ends_with(part) {
                return false;
            }
            rest = &rest[..rest.len() - part.len()];
        } else if let Some(pos) = rest.find(part) {
            rest = &rest[pos + part.len()..];
        } else {
            return false;
        }
    }
    true
}

/// Whether `path` exists on disk — used by ingestors to decide between the
/// `DISCOVERED` and `UNSEEN` fallback statuses when nothing else applies.
pub fn path_exists(path: &str) -> bool {
    !path.is_empty() && Path::new(path).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_pattern_with_no_wildcard_is_exact() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("plan.json");
        std::fs::write(&file, "{}").unwrap();
        let found = expand(file.to_str().unwrap());
        assert_eq!(found, vec![file]);
    }

    #[test]
    fn missing_literal_path_yields_no_matches() {
        assert!(expand("/definitely/does/not/exist/plan.json").is_empty());
    }

    #[test]
    fn star_matches_within_segment() {
        assert!(glob_match("benches/*.txt", "benches/foo.txt"));
        assert!(!glob_match("benches/*.txt", "benches/sub/foo.txt"));
    }

    #[test]
    fn double_star_matches_any_depth() {
        assert!(glob_match("target/**/plan.json", "target/plan.json"));
        assert!(glob_match(
            "target/**/plan.json",
            "target/a/b/c/plan.json"
        ));
        assert!(!glob_match("target/**/plan.json", "target/plan.txt"));
    }

    #[test]
    fn expand_finds_nested_matches() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("runs/a")).unwrap();
        std::fs::create_dir_all(dir.path().join("runs/b")).unwrap();
        std::fs::write(dir.path().join("runs/a/plan.json"), "{}").unwrap();
        std::fs::write(dir.path().join("runs/b/plan.json"), "{}").unwrap();
        std::fs::write(dir.path().join("runs/b/other.json"), "{}").unwrap();
        let pattern = format!("{}/runs/**/plan.json", dir.path().to_str().unwrap());
        let found = expand(&pattern);
        assert_eq!(found.len(), 2);
    }
}
