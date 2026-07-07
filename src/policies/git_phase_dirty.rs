use super::{CicdPolicy, PolicyResult};

pub struct GitPhaseDirtyPolicy;

impl CicdPolicy for GitPhaseDirtyPolicy {
    fn name(&self) -> &'static str {
        "git_phase_dirty"
    }
    fn enabled(&self) -> bool {
        true
    }
    fn evaluate(&self, state: &crate::engine::EngineState) -> PolicyResult {
        let is_dirty =
            !state.git_phase.dirty_files.is_empty() || !state.git_phase.untracked_files.is_empty();
        let (verdict, rec) = if is_dirty {
            (
                "alert",
                Some("working tree is dirty — commit or stash changes before CI run".into()),
            )
        } else {
            ("pass", None)
        };
        PolicyResult {
            verdict: verdict.into(),
            recommendation: rec,
        }
    }
}
