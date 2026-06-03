//! TargetHygieneAnalyzer — raises CICD-TARGET-001 and CICD-TARGET-002.
//!
//! Reports target directory growth warnings at 15 GB and prune recommendations at 20 GB.

use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding};
use cargo_cicd_core::workspace::snapshot::WorkspaceSnapshot;

use super::CicdAnalyzer;

const WARN_BYTES: u64 = 15 * 1024 * 1024 * 1024; // 15 GB
const PRUNE_BYTES: u64 = 20 * 1024 * 1024 * 1024; // 20 GB

/// Recursively compute directory size in bytes (best-effort; skips unreadable entries).
fn dir_size_bytes(path: &std::path::Path) -> u64 {
    let mut total: u64 = 0;
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            total += dir_size_bytes(&p);
        } else if let Ok(meta) = p.metadata() {
            total += meta.len();
        }
    }
    total
}

fn gb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

/// Analyzes the target directory for hygiene issues.
pub struct TargetHygieneAnalyzer;

impl CicdAnalyzer for TargetHygieneAnalyzer {
    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
        let mut findings = Vec::new();

        let target_dir = snapshot.root.join("target");
        if !target_dir.is_dir() {
            return findings;
        }

        let size = dir_size_bytes(&target_dir);
        let target_str = target_dir.to_string_lossy().into_owned();

        if size >= PRUNE_BYTES {
            // CICD-TARGET-002: target prune recommended (>= 20 GB).
            findings.push(CicdFinding::new(
                CicdCode::TargetDirOversize,
                target_str,
                "cargo clean",
                vec!["cargo clean".to_string()],
                format!(
                    "target/ directory is {:.1} GB (>= 20 GB threshold). \
                     Run `cargo clean` to reclaim disk space.",
                    gb(size)
                ),
            ));
        } else if size >= WARN_BYTES {
            // CICD-TARGET-001: target directory growth warning (>= 15 GB).
            findings.push(CicdFinding::new(
                CicdCode::TargetDirOversize,
                target_str,
                "cargo clean",
                vec!["cargo clean".to_string()],
                format!(
                    "target/ directory is {:.1} GB (>= 15 GB). \
                     Consider running `cargo clean` before it grows further.",
                    gb(size)
                ),
            ));
        }

        findings
    }

    fn name(&self) -> &'static str {
        "TargetHygieneAnalyzer"
    }
}
