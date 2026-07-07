//! Autonomic policy: branch behind remote blocks close.
use super::{CicdPolicy, PolicyResult};

pub struct BranchBehindPolicy;

impl CicdPolicy for BranchBehindPolicy {
    fn name(&self) -> &'static str {
        "branch_behind"
    }

    fn enabled(&self) -> bool {
        true
    }

    fn evaluate(&self, state: &crate::engine::EngineState) -> PolicyResult {
        let behind_count = if state.git_phase.behind > 0 {
            Some(state.git_phase.behind as usize)
        } else {
            None
        };

        let (verdict, rec) = match behind_count {
            Some(n) if n > 0 => (
                "alert",
                Some(format!(
                    "branch is {} commit(s) behind remote — run 'git pull --rebase' to sync",
                    n
                )),
            ),
            Some(_) => ("pass", None),
            // git failed or no upstream configured — treat gracefully
            None => ("pass", None),
        };

        PolicyResult {
            verdict: verdict.into(),
            recommendation: rec,
        }
    }
}
