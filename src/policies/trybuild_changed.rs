use super::{CicdPolicy, PolicyMode, PolicyResult};
use crate::adapters::ChangedFileDetector;

pub struct TrybuildChangedPolicy;

impl CicdPolicy for TrybuildChangedPolicy {
    fn name(&self) -> &'static str { "trybuild_changed" }
    fn enabled(&self) -> bool { true }
    fn mode(&self) -> PolicyMode { PolicyMode::Suggest }
    fn evaluate(&self) -> PolicyResult {
        let changed = ChangedFileDetector::changed_rs_files("origin/main");
        let fixtures: Vec<_> = changed.iter().filter(|f| ChangedFileDetector::is_trybuild_fixture(f)).collect();
        let (verdict, rec) = if fixtures.is_empty() {
            ("pass", None)
        } else {
            ("warn", Some(format!("{} trybuild fixture(s) changed — run 'cargo cicd trybuild changed' to test selectively", fixtures.len())))
        };
        PolicyResult { name: self.name().into(), enabled: true, mode: "suggest".into(), verdict: verdict.into(), recommendation: rec, event_kind: "trybuild_changed".into() }
    }
}
