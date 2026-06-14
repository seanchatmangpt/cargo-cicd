//! Autonomic policy: branch behind remote blocks close.
use super::{CicdPolicy, PolicyMode, PolicyResult};
use std::process::Command;

pub struct BranchBehindPolicy;

impl CicdPolicy for BranchBehindPolicy {
    fn name(&self) -> &'static str {
        "branch_behind"
    }

    fn enabled(&self) -> bool {
        true
    }

    fn mode(&self) -> PolicyMode {
        PolicyMode::Suggest
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
            name: self.name().into(),
            enabled: true,
            mode: "suggest".into(),
            verdict: verdict.into(),
            recommendation: rec,
            event_kind: "branch_behind".into(),
        }
    }
}

/// Run `git rev-list --count HEAD..@{u}` and parse the result.
///
/// Returns `None` when git is unavailable or no upstream is configured.
fn count_commits_behind() -> Option<usize> {
    let output = Command::new("git")
        .args(["rev-list", "--count", "HEAD..@{u}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    stdout.trim().parse::<usize>().ok()
}
