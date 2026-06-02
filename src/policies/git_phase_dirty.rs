use super::{CicdPolicy, PolicyMode, PolicyResult};
use crate::adapters::GitStatusAdapter;

pub struct GitPhaseDirtyPolicy;

impl CicdPolicy for GitPhaseDirtyPolicy {
    fn name(&self) -> &'static str { "git_phase_dirty" }
    fn enabled(&self) -> bool { true }
    fn mode(&self) -> PolicyMode { PolicyMode::Suggest }
    fn evaluate(&self) -> PolicyResult {
        let is_dirty = GitStatusAdapter::is_dirty();
        let (verdict, rec) = if is_dirty {
            ("alert", Some("working tree is dirty — commit or stash changes before CI run".into()))
        } else {
            ("pass", None)
        };
        PolicyResult { name: self.name().into(), enabled: true, mode: "suggest".into(), verdict: verdict.into(), recommendation: rec, event_kind: "git_phase_dirty".into() }
    }
}
