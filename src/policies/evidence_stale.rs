//! Autonomic policy: stale evidence blocks close.
use super::{CicdPolicy, PolicyMode, PolicyResult};

pub struct EvidenceStalePolicy;

impl CicdPolicy for EvidenceStalePolicy {
    fn name(&self) -> &'static str {
        "evidence_stale"
    }

    fn enabled(&self) -> bool {
        true
    }

    fn mode(&self) -> PolicyMode {
        PolicyMode::Suggest
    }

    fn evaluate(&self, state: &crate::engine::EngineState) -> PolicyResult {
        let has_changes = state.changed_files.total_changed > 0;

        // Check whether evidence directory has recent output.
        let evidence_dir = std::path::Path::new("target/cargo-cicd/evidence");
        let evidence_fresh = if evidence_dir.exists() {
            // Evidence is considered fresh when either the OCEL 2.0 log or the legacy
            // XES file exists. OCEL is the primary format; XES is backward-compat.
            let ocel = evidence_dir.join("events.ocel.json");
            let xes = evidence_dir.join("events.xes");
            ocel.exists() || xes.exists()
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
                Some("source changes detected — verify evidence is current before closing".into()),
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
