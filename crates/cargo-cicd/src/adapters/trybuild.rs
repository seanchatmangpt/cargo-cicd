use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

/// Find trybuild fixture files (under `tests/ui/`) that changed since `base`.
///
/// Uses `git diff --name-only <base>` and filters to `.rs` files under `tests/ui/`.
pub fn find_changed_trybuild_fixtures(base: &str) -> Result<Vec<PathBuf>> {
    let out = Command::new("git")
        .args(["diff", "--name-only", base])
        .output()?;
    let fixtures = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter(|l| {
            let p = l.to_lowercase();
            (p.contains("tests/ui/") || p.contains("tests\\ui\\")) && p.ends_with(".rs")
        })
        .map(|l| PathBuf::from(l.trim()))
        .collect();
    Ok(fixtures)
}
