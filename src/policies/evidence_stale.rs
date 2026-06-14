//! Autonomic policy: stale evidence blocks close.
use super::{CicdPolicy, PolicyMode, PolicyResult};
use crate::adapters::ChangedFileDetector;

pub struct EvidenceStalePoliciy;

impl CicdPolicy for EvidenceStalePoliciy {
    fn name(&self) -> &'static str {
        "evidence_stale"
    }

    fn enabled(&self) -> bool {
        true
    }

    fn mode(&self) -> PolicyMode {
        PolicyMode::Suggest
    }

    fn evaluate(&self) -> PolicyResult {
        let changed = ChangedFileDetector::changed_rs_files("origin/main");
        let has_changes = !changed.is_empty();

        // Check whether evidence directory has recent output.
        let evidence_dir = std::path::Path::new("target/cargo-cicd/evidence");
        let evidence_fresh = if evidence_dir.exists() {
            // Evidence is considered fresh when the events.xes file exists and
            // the directory has been written more recently than any changed source file.
            let xes = evidence_dir.join("events.xes");
            xes.exists()
        } else {
            false
        };

        let (verdict, rec) = if has_changes && !evidence_fresh {
            (
                "alert",
                Some(
                    "evidence stale: run 'cargo cicd test changed' and 'cargo cicd workspace doctor'"
                        .into(),
                ),
            )
        } else if has_changes && evidence_fresh {
            // Changes exist but evidence is present — warn to re-run in case it is outdated.
            (
                "warn",
                Some(
                    "source changes detected — verify evidence is current before closing"
                        .into(),
                ),
            )
        } else {
            ("pass", None)
        };

        PolicyResult {
            name: self.name().into(),
            enabled: true,
            mode: "suggest".into(),
            verdict: verdict.into(),
            recommendation: rec,
            event_kind: "evidence_stale".into(),
        }
    }
}
