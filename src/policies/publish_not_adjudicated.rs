//! Autonomic policy: publish requires adjudicated receipt.
use super::{CicdPolicy, PolicyResult};
use std::path::Path;

pub struct PublishNotAdjudicatedPolicy;

impl CicdPolicy for PublishNotAdjudicatedPolicy {
    fn name(&self) -> &'static str {
        "publish_not_adjudicated"
    }

    fn enabled(&self) -> bool {
        true
    }

    fn evaluate(&self, _state: &crate::engine::EngineState) -> PolicyResult {
        let receipt_path = Path::new("target/cargo-cicd/evidence/receipts/latest.json");
        let evidence_dir = Path::new("target/cargo-cicd/evidence");

        let (verdict, rec) = if !receipt_path.exists() {
            (
                "alert",
                Some(
                    "no adjudicated receipt found — run 'cargo cicd evidence doctor' before publish"
                        .into(),
                ),
            )
        } else {
            // Receipt exists — check whether it is stale relative to the evidence directory.
            let receipt_stale = is_receipt_stale(receipt_path, evidence_dir);
            if receipt_stale {
                (
                    "warn",
                    Some(
                        "receipt exists but may be stale — re-run 'cargo cicd evidence doctor' to refresh"
                            .into(),
                    ),
                )
            } else {
                ("pass", None)
            }
        };

        PolicyResult {
            verdict: verdict.into(),
            recommendation: rec,
        }
    }
}

/// Return `true` when the `latest.json` receipt is older than the most
/// recently modified file in the evidence directory.
fn is_receipt_stale(receipt: &Path, evidence_dir: &Path) -> bool {
    let receipt_mtime = mtime(receipt);
    let Some(r_mtime) = receipt_mtime else {
        return false;
    };

    // Walk the evidence directory; if any sibling file is newer than the
    // receipt, the receipt is considered stale.
    let Ok(entries) = std::fs::read_dir(evidence_dir) else {
        return false;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        // Skip the receipt itself.
        if path == receipt {
            continue;
        }
        if let Some(m) = mtime(&path) {
            if m > r_mtime {
                return true;
            }
        }
    }

    false
}

fn mtime(path: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}
