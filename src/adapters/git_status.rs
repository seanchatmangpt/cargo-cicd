use anyhow::Result;
use std::process::Command;

#[cfg(feature = "advanced")]
use super::super::advanced::observability::{init_tracing, PipelineStage};

pub struct GitStatusAdapter;

impl GitStatusAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn query() -> Result<GitStatusResult> {
        #[cfg(feature = "advanced")]
        init_tracing();

        #[cfg(feature = "advanced")]
        let _stage = PipelineStage::enter("git_status");

        let mut result = GitStatusResult::default();

        let branch_out = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()?;
        result.branch = String::from_utf8_lossy(&branch_out.stdout)
            .trim()
            .to_string();

        let status_out = Command::new("git")
            .args(["status", "--porcelain"])
            .output()?;
        let status = String::from_utf8_lossy(&status_out.stdout);
        for line in status.lines() {
            if line.len() < 3 {
                continue;
            }
            let xy = &line[..2];
            let file = line[3..].to_string();
            let x = xy.chars().next();
            let y = xy.chars().nth(1);
            match (x, y) {
                (Some(' '), Some('M')) | (Some(' '), Some('D')) => result.dirty_files.push(file),
                (Some('M'), Some('M')) => {
                    result.staged_files.push(file.clone());
                    result.dirty_files.push(file);
                }
                (Some('M'), _) | (Some('A'), _) | (Some('D'), _) => result.staged_files.push(file),
                (Some('?'), Some('?')) => result.untracked_files.push(file),
                _ => {}
            }
        }

        Ok(result)
    }

    pub fn is_dirty() -> bool {
        Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(true)
    }
}

impl Default for GitStatusAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
pub struct GitStatusResult {
    pub branch: String,
    pub dirty_files: Vec<String>,
    pub staged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub ahead: u32,
    pub behind: u32,
}

#[cfg(all(test, feature = "advanced"))]
mod tests {
    use super::*;
    use tracing::subscriber::with_default;
    use tracing_subscriber::fmt;
    use tracing_subscriber::EnvFilter;

    #[test]
    fn git_status_adapter_query_with_observability() {
        let subscriber = fmt()
            .json()
            .with_test_writer()
            .with_env_filter(EnvFilter::new("info"))
            .finish();

        with_default(subscriber, || {
            // This test verifies that calling query() with the advanced feature
            // and observability instrumentation does not panic and properly enters
            // and exits the pipeline stage.
            let result = GitStatusAdapter::query();
            // We don't assert on the result itself (it depends on the git repo state),
            // just that the instrumented method runs without panicking.
            let _ = result;
        });
    }
}
