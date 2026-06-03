//! PublishAnalyzer — raises CICD-PUBLISH-{001,002,003} based on publish readiness.
//!
//! | Code               | Condition                                                        |
//! |--------------------|------------------------------------------------------------------|
//! | CICD-PUBLISH-001   | No cicd.toml found — publish state is undefined                  |
//! | CICD-PUBLISH-002   | Dry-run marker present but no matching receipt exists            |
//! | CICD-PUBLISH-003   | Package changed after dry-run (Cargo.toml mtime > receipt mtime) |

use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding};
use cargo_cicd_core::publish::readiness::PublishReadiness;
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;

use super::CicdAnalyzer;

/// Checks publish readiness across all three PUBLISH codes.
pub struct PublishAnalyzer;

impl CicdAnalyzer for PublishAnalyzer {
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
        let mut findings = Vec::new();
        let readiness = PublishReadiness::from_workspace(&snapshot.root);

        // CICD-PUBLISH-001: no cicd.toml found — publish state is undefined.
        if !readiness.cicd_toml_exists {
            findings.push(CicdFinding::new(
                CicdCode::PublishNoCicdToml,
                "workspace root",
                "cicd.toml",
                vec!["cargo cicd init".to_string()],
                "No cicd.toml found in the workspace root. \
                 Initialize cicd.toml before attempting to publish.",
            ));
            // No further publish checks are meaningful without cicd.toml.
            return findings;
        }

        // CICD-PUBLISH-002: dry-run marker present but no matching receipt.
        if readiness.dry_run_marker.is_some() && !readiness.dry_run_has_receipt {
            let marker = readiness.dry_run_marker.as_deref().unwrap_or(".dry-run");
            findings.push(CicdFinding::new(
                CicdCode::PublishDryRunWithoutReceipt,
                marker,
                "receipts/",
                vec!["cargo cicd publish".to_string()],
                format!(
                    "Dry-run marker '{}' is present but no valid receipt exists in receipts/. \
                     Obtain a wpm-confirmed receipt before publishing.",
                    marker
                ),
            ));
        }

        // CICD-PUBLISH-003: package changed after dry-run (Cargo.toml mtime > receipt mtime).
        if let Some(finding) = check_package_changed_after_dry_run(snapshot, &readiness) {
            findings.push(finding);
        }

        findings
    }

    fn name(&self) -> &'static str {
        "PublishAnalyzer"
    }
}

/// Returns CICD-PUBLISH-003 if Cargo.toml is newer than the latest receipt.
fn check_package_changed_after_dry_run(
    snapshot: &WorkspaceSnapshot,
    readiness: &PublishReadiness,
) -> Option<CicdFinding> {
    // Only relevant when a dry-run marker and at least one receipt exist.
    if !readiness.dry_run_has_receipt {
        return None;
    }

    let cargo_toml = snapshot.root.join("Cargo.toml");
    let cargo_mtime = std::fs::metadata(&cargo_toml)
        .and_then(|m| m.modified())
        .ok()?;

    let receipts_dir = snapshot.root.join("receipts");
    let latest_receipt_mtime = std::fs::read_dir(&receipts_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter_map(|e| std::fs::metadata(e.path()).and_then(|m| m.modified()).ok())
        .max()?;

    if cargo_mtime > latest_receipt_mtime {
        Some(CicdFinding::new(
            CicdCode::PublishNoReceipt,
            "Cargo.toml",
            "receipts/",
            vec!["cargo cicd publish".to_string()],
            "Cargo.toml was modified after the last dry-run receipt was issued. \
             The package has changed since the dry-run; a new receipt is required \
             before publishing.",
        ))
    } else {
        None
    }
}
