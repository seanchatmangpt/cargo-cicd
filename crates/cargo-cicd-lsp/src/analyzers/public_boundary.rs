//! PublicBoundaryAnalyzer — raises CICD-PUBLIC-{001,002}.
//!
//! | Code             | Condition                                                              |
//! |------------------|------------------------------------------------------------------------|
//! | CICD-PUBLIC-001  | Private/forbidden term found in docs/ README.md or src/ help text     |
//! | CICD-PUBLIC-002  | Public boundary scan file stale (scan not run recently)               |

use std::time::{Duration, SystemTime};

use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding};
use cargo_cicd_core::public_boundary::scan::{scan_dir, scan_file};
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;

use super::CicdAnalyzer;

/// Maximum age of a scan stamp before CICD-PUBLIC-002 is raised (24 hours).
const SCAN_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// Canonical stamp file written by `cargo cicd boundary scan`.
const SCAN_STAMP_REL: &str = "target/cargo-cicd/boundary-scan.stamp";

/// Scans public-facing documentation surfaces for private/forbidden terms.
pub struct PublicBoundaryAnalyzer;

impl CicdAnalyzer for PublicBoundaryAnalyzer {
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
        let mut findings = Vec::new();

        // ── CICD-PUBLIC-001: forbidden term scan ─────────────────────────────

        // Scan README.md at workspace root.
        let readme = snapshot.root.join("README.md");
        if readme.exists() {
            for violation in scan_file(&readme) {
                findings.push(
                    CicdFinding::new(
                        CicdCode::PublicPrivateTermLeak,
                        violation.file.clone(),
                        "source content",
                        vec!["cargo cicd boundary scan".to_string()],
                        format!(
                            "Forbidden term '{}' found at line {} in {}: {}",
                            violation.term, violation.line, violation.file, violation.context
                        ),
                    )
                    .at_uri(violation.file),
                );
            }
        }

        // Scan docs/**/*.md
        let docs_dir = snapshot.root.join("docs");
        if docs_dir.is_dir() {
            for violation in scan_dir(&docs_dir, &["md"]) {
                findings.push(
                    CicdFinding::new(
                        CicdCode::PublicPrivateTermLeak,
                        violation.file.clone(),
                        "source content",
                        vec!["cargo cicd boundary scan".to_string()],
                        format!(
                            "Forbidden term '{}' found at line {} in {}: {}",
                            violation.term, violation.line, violation.file, violation.context
                        ),
                    )
                    .at_uri(violation.file),
                );
            }
        }

        // Scan src/**/*.rs for help-text strings containing forbidden terms.
        let src_dir = snapshot.root.join("src");
        if src_dir.is_dir() {
            for violation in scan_dir(&src_dir, &["rs"]) {
                findings.push(
                    CicdFinding::new(
                        CicdCode::PublicPrivateTermLeak,
                        violation.file.clone(),
                        "source content",
                        vec!["cargo cicd boundary scan".to_string()],
                        format!(
                            "Forbidden term '{}' in help text at line {} in {}: {}",
                            violation.term, violation.line, violation.file, violation.context
                        ),
                    )
                    .at_uri(violation.file),
                );
            }
        }

        // ── CICD-PUBLIC-002: stale boundary scan ─────────────────────────────

        if let Some(finding) = check_scan_staleness(snapshot) {
            findings.push(finding);
        }

        findings
    }

    fn name(&self) -> &'static str {
        "PublicBoundaryAnalyzer"
    }
}

/// Returns CICD-PUBLIC-002 when the boundary-scan stamp is absent or older than
/// [`SCAN_MAX_AGE`].
fn check_scan_staleness(snapshot: &WorkspaceSnapshot) -> Option<CicdFinding> {
    let stamp = snapshot.root.join(SCAN_STAMP_REL);

    let stamp_age: Option<Duration> = std::fs::metadata(&stamp)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|mtime| SystemTime::now().duration_since(mtime).ok());

    let stale = match stamp_age {
        None => true, // stamp missing or unreadable
        Some(age) => age > SCAN_MAX_AGE,
    };

    if stale {
        let msg = if stamp.exists() {
            format!(
                "Public boundary scan stamp '{}' is older than {} hours. \
                 Re-run `cargo cicd boundary scan` to refresh.",
                SCAN_STAMP_REL,
                SCAN_MAX_AGE.as_secs() / 3600,
            )
        } else {
            format!(
                "Public boundary scan stamp '{}' is absent. \
                 Run `cargo cicd boundary scan` to establish a baseline.",
                SCAN_STAMP_REL,
            )
        };

        Some(CicdFinding::new(
            CicdCode::BoundaryPublicApiLeak,
            SCAN_STAMP_REL,
            "boundary scan",
            vec!["cargo cicd boundary scan".to_string()],
            msg,
        ))
    } else {
        None
    }
}
