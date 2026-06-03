//! ChangedTestsAnalyzer — raises CICD-TEST-001 through CICD-TEST-003.
//!
//! - CICD-TEST-001: changed test files exist but evidence is stale.
//! - CICD-TEST-002: trybuild fixture changed.
//! - CICD-TEST-003: .stderr receipt for trybuild is stale.

use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding};
use cargo_cicd_core::evidence::freshness::FreshnessVerdict;
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;

use super::CicdAnalyzer;

/// Analyzes whether changed test files and trybuild fixtures have fresh evidence.
pub struct ChangedTestsAnalyzer;

/// Collect all `tests/ui/` `.rs` fixture files under `root`.
fn trybuild_fixtures(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let ui_dir = root.join("tests").join("ui");
    if !ui_dir.is_dir() {
        return Vec::new();
    }
    collect_rs_files(&ui_dir)
}

fn collect_rs_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return result;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            result.extend(collect_rs_files(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            result.push(path);
        }
    }
    result
}

/// Check whether a trybuild `.rs` fixture has a corresponding `.stderr` file and whether
/// the `.stderr` is older than the `.rs`.
fn stderr_receipt_stale(fixture: &std::path::Path) -> (bool /* missing */, bool /* stale */) {
    let stderr_path = fixture.with_extension("stderr");
    if !stderr_path.exists() {
        return (true, false);
    }
    let fix_mtime = fixture.metadata().ok().and_then(|m| m.modified().ok());
    let err_mtime = stderr_path.metadata().ok().and_then(|m| m.modified().ok());
    match (fix_mtime, err_mtime) {
        (Some(fm), Some(em)) => (false, em < fm),
        _ => (false, false),
    }
}

impl CicdAnalyzer for ChangedTestsAnalyzer {
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
        let mut findings = Vec::new();

        // CICD-TEST-001: changed test files exist but evidence is stale.
        // Proxy: any `tests/` `.rs` file newer than the evidence.
        let tests_dir = snapshot.root.join("tests");
        if tests_dir.is_dir() {
            let evidence_dir = snapshot
                .root
                .join("target")
                .join("cargo-cicd")
                .join("evidence");
            let evidence_mtime = evidence_dir.metadata().ok().and_then(|m| m.modified().ok());

            let mut stale_evidence = false;
            if snapshot.evidence_state.freshness == FreshnessVerdict::Stale
                || !snapshot.evidence_state.exists
            {
                // Evidence is already known-stale — check if test files exist at all.
                stale_evidence = tests_dir.is_dir();
            } else if let Some(ev_mt) = evidence_mtime {
                // Check if any test file is newer than the evidence dir.
                stale_evidence = collect_rs_files(&tests_dir).iter().any(|f| {
                    f.metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(|fmt| fmt > ev_mt)
                        .unwrap_or(false)
                });
            }

            if stale_evidence {
                findings.push(CicdFinding::new(
                    CicdCode::TestsStaleMapping,
                    tests_dir.to_string_lossy().as_ref(),
                    "cargo cicd test changed",
                    vec!["cargo cicd test changed".to_string()],
                    "Changed test files exist but process evidence is stale. \
                     Re-run the manufacturing pipeline to refresh evidence.",
                ));
            }
        }

        // CICD-TEST-002 and CICD-TEST-003: trybuild fixtures.
        let fixtures = trybuild_fixtures(&snapshot.root);

        for fixture in &fixtures {
            let fixture_str = fixture.to_string_lossy().into_owned();

            // CICD-TEST-002: trybuild fixture changed (newer than evidence).
            let evidence_dir = snapshot
                .root
                .join("target")
                .join("cargo-cicd")
                .join("evidence");
            let evidence_mtime = evidence_dir.metadata().ok().and_then(|m| m.modified().ok());

            if let Some(ev_mt) = evidence_mtime {
                let fix_mtime = fixture.metadata().ok().and_then(|m| m.modified().ok());
                if let Some(fm) = fix_mtime {
                    if fm > ev_mt {
                        findings.push(CicdFinding::new(
                            CicdCode::TestsImpactUnknown,
                            fixture_str.clone(),
                            "cargo test --test ui_tests -- --ignored",
                            vec!["cargo test --test ui_tests -- --ignored".to_string()],
                            format!(
                                "Trybuild fixture '{}' has changed since the last evidence run. \
                                 Re-run the ui_tests gate.",
                                fixture_str
                            ),
                        ));
                    }
                }
            }

            // CICD-TEST-003: .stderr receipt for trybuild is stale.
            let (missing, stale) = stderr_receipt_stale(fixture);
            if missing {
                findings.push(CicdFinding::new(
                    CicdCode::TestsStaleMapping,
                    fixture_str.clone(),
                    "cargo test --test ui_tests -- --ignored",
                    vec!["cargo test --test ui_tests -- --ignored".to_string()],
                    format!(
                        "Trybuild fixture '{}' has no corresponding .stderr receipt. \
                         Run the ui_tests gate to generate expected compiler output.",
                        fixture_str
                    ),
                ));
            } else if stale {
                findings.push(CicdFinding::new(
                    CicdCode::TestsStaleMapping,
                    fixture_str.clone(),
                    "cargo test --test ui_tests -- --ignored",
                    vec!["cargo test --test ui_tests -- --ignored".to_string()],
                    format!(
                        ".stderr receipt for trybuild fixture '{}' is older than the fixture itself. \
                         Re-run the ui_tests gate to refresh the expected compiler output.",
                        fixture_str
                    ),
                ));
            }
        }

        findings
    }

    fn name(&self) -> &'static str {
        "ChangedTestsAnalyzer"
    }
}
