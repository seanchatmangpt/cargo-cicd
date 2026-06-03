//! RenderedSurfaceAnalyzer — raises CICD-GGEN-001 through CICD-GGEN-003.
//!
//! Scans `docs/` and `README.md` for ggen and custom region markers.

use std::path::Path;
use std::time::SystemTime;

use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding};
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;

use super::CicdAnalyzer;

/// Analyzes rendered documentation surfaces for ggen drift and missing custom regions.
pub struct RenderedSurfaceAnalyzer;

/// Returns the modification time of a file, or `None` on failure.
fn mtime(path: &Path) -> Option<SystemTime> {
    path.metadata().ok().and_then(|m| m.modified().ok())
}

/// Collect candidate rendered surface files: docs/**/*.md and README.md.
fn rendered_candidates(root: &Path) -> Vec<std::path::PathBuf> {
    let mut candidates = Vec::new();

    let readme = root.join("README.md");
    if readme.exists() {
        candidates.push(readme);
    }

    let docs_dir = root.join("docs");
    if docs_dir.is_dir() {
        if let Ok(entries) = walkdir_md(&docs_dir) {
            candidates.extend(entries);
        }
    }

    candidates
}

/// Walk a directory and return all `.md` files.
fn walkdir_md(dir: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut result = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Ok(mut sub) = walkdir_md(&path) {
                result.append(&mut sub);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            result.push(path);
        }
    }
    Ok(result)
}

impl CicdAnalyzer for RenderedSurfaceAnalyzer {
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
        let mut findings = Vec::new();

        let ggen_toml = snapshot.root.join("ggen.toml");
        let has_ggen_toml = ggen_toml.exists();

        let candidates = rendered_candidates(&snapshot.root);

        for file in &candidates {
            let content = match std::fs::read_to_string(file) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let file_str = file.to_string_lossy().into_owned();

            // CICD-GGEN-001: ggen-rendered file stale.
            // Condition: ggen.toml present and the rendered file is older than ggen.toml.
            if has_ggen_toml {
                let ggen_mtime = mtime(&ggen_toml);
                let file_mtime = mtime(file);
                if let (Some(gmt), Some(fmt)) = (ggen_mtime, file_mtime) {
                    if fmt < gmt {
                        findings.push(CicdFinding::new(
                            CicdCode::GgenDriftDetected,
                            file_str.clone(),
                            "ggen",
                            vec!["ggen".to_string()],
                            format!(
                                "Rendered file '{}' is older than ggen.toml — re-run `ggen` to regenerate.",
                                file_str
                            ),
                        ));
                    }
                }
            }

            // CICD-GGEN-002: rendered surface drift — ggen block changed outside ggen.
            // Detect by looking for BEGIN ggen: markers without matching END markers.
            let ggen_begins = content
                .lines()
                .filter(|l| l.contains("BEGIN ggen:"))
                .count();
            let ggen_ends = content.lines().filter(|l| l.contains("END ggen:")).count();

            if ggen_begins > 0 && ggen_begins != ggen_ends {
                findings.push(CicdFinding::new(
                    CicdCode::GgenRenderedSurfaceDrift,
                    file_str.clone(),
                    "ggen",
                    vec!["ggen".to_string()],
                    format!(
                        "Rendered file '{}' has {} BEGIN ggen: marker(s) but {} END ggen: marker(s). \
                         ggen block changed outside ggen — re-run `ggen`.",
                        file_str, ggen_begins, ggen_ends
                    ),
                ));
            }

            // CICD-GGEN-003: custom region missing from rendered file.
            // A rendered file with a ggen block should also have a custom region guard.
            if ggen_begins > 0 {
                let has_custom_begin = content.lines().any(|l| l.contains("BEGIN custom:"));
                if !has_custom_begin {
                    findings.push(CicdFinding::new(
                        CicdCode::GgenCustomRegionMissing,
                        file_str.clone(),
                        "ggen",
                        vec!["ggen".to_string()],
                        format!(
                            "Rendered file '{}' has a ggen block but no BEGIN custom: region guard. \
                             Re-run `ggen` or restore the guard so custom edits survive regen.",
                            file_str
                        ),
                    ));
                }
            }
        }

        findings
    }

    fn name(&self) -> &'static str {
        "RenderedSurfaceAnalyzer"
    }
}
