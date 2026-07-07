use super::{CicdPolicy, PolicyResult};

pub struct TrybuildChangedPolicy;

impl CicdPolicy for TrybuildChangedPolicy {
    fn name(&self) -> &'static str {
        "trybuild_changed"
    }
    fn enabled(&self) -> bool {
        true
    }
    fn evaluate(&self, state: &crate::engine::EngineState) -> PolicyResult {
        let fixture_count = state.trybuild.changed_fixtures.len();
        let (verdict, rec) = if fixture_count == 0 {
            ("pass", None)
        } else {
            ("warn", Some(format!("{} trybuild fixture(s) changed — run 'cargo cicd trybuild changed' to test selectively", fixture_count)))
        };
        PolicyResult {
            verdict: verdict.into(),
            recommendation: rec,
        }
    }
}
