//! Portable diff-base reference resolution.
//!
//! Resolves the "what did we change since?" ref through a fallback chain so a
//! fresh clone or detached HEAD fails with a typed error instead of a
//! mysterious empty diff or a hard failure on a hardcoded `origin/main`.

use std::path::Path;
use std::process::Command;

/// Default fallback chain, in resolution order.
pub const DEFAULT_BASE_CHAIN: &[&str] = &["origin/main", "main"];

/// Why a base ref could not be resolved.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BaseRefError {
    /// An explicitly configured ref does not exist in this repository.
    #[error("configured base ref `{ref_}` does not resolve in this repository (fresh clone? detached HEAD? fetch it first)")]
    ConfiguredRefMissing { ref_: String },
    /// No configured ref and none of the default candidates resolve.
    #[error(
        "no base ref resolved (tried {tried:?}); run `git fetch` or configure `test.changed.base` in cargo-cicd.toml"
    )]
    NoBaseResolvable { tried: Vec<String> },
    /// `git` itself could not be executed (missing binary, not a repo, ...).
    #[error("git invocation failed in {root}: {detail}")]
    GitFailed { root: String, detail: String },
}

/// Resolve the diff base ref.
///
/// Chain: explicit config value > `origin/main` > `main` > first commit.
/// The first commit fallback keeps single-branch / shallow-history repos
/// working; only when *nothing* resolves (fresh clone with no fetched
/// branches and no commits) is a typed error returned.
pub fn resolve_base_ref(explicit: Option<&str>, root: &Path) -> Result<String, BaseRefError> {
    if let Some(cfg) = explicit {
        if ref_resolves(cfg, root)? {
            return Ok(cfg.to_string());
        }
        return Err(BaseRefError::ConfiguredRefMissing {
            ref_: cfg.to_string(),
        });
    }

    let mut tried = Vec::new();
    for candidate in DEFAULT_BASE_CHAIN {
        if ref_resolves(candidate, root)? {
            return Ok((*candidate).to_string());
        }
        tried.push((*candidate).to_string());
    }

    if first_commit(root)?.is_some() {
        return Ok("HEAD".to_string());
    }
    Err(BaseRefError::NoBaseResolvable { tried })
}

/// True when `git rev-parse --verify --quiet <ref>` succeeds.
fn ref_resolves(ref_: &str, root: &Path) -> Result<bool, BaseRefError> {
    let out = git(root, &["rev-parse", "--verify", "--quiet", ref_])?;
    Ok(out.status.success())
}

/// SHA of the root commit, if the repo has any history at all.
fn first_commit(root: &Path) -> Result<Option<String>, BaseRefError> {
    let out = git(root, &["rev-list", "--max-parents=0", "HEAD"])?;
    if !out.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.lines().next().map(|l| l.trim().to_string()))
}

fn git(root: &Path, args: &[&str]) -> Result<std::process::Output, BaseRefError> {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| BaseRefError::GitFailed {
            root: root.display().to_string(),
            detail: e.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command as SysCommand;

    fn git_init(dir: &Path) {
        let ok = SysCommand::new("git")
            .args(["init", "-b", "main"])
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(ok.status.success(), "git init failed");
    }

    fn commit_all(dir: &Path, msg: &str) {
        SysCommand::new("git")
            .args(["add", "-A"])
            .current_dir(dir)
            .output()
            .unwrap()
            .status
            .success()
            .then_some(())
            .unwrap();
        let c = SysCommand::new("git")
            .args([
                "-c",
                "user.email=t@t",
                "-c",
                "user.name=t",
                "commit",
                "-m",
                msg,
            ])
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(c.status.success(), "commit failed: {}", msg);
    }

    #[test]
    fn fresh_clone_local_main_resolves_before_head_fallback() {
        let dir = tempfile::tempdir().unwrap();
        git_init(dir.path());
        std::fs::write(dir.path().join("a.txt"), "x").unwrap();
        commit_all(dir.path(), "root");
        // No origin/main, no remote: local `main` (from init -b main) resolves.
        let resolved = resolve_base_ref(None, dir.path()).unwrap();
        assert_eq!(resolved, "main");
    }

    #[test]
    fn detached_head_only_history_uses_head_fallback() {
        let dir = tempfile::tempdir().unwrap();
        git_init(dir.path());
        std::fs::write(dir.path().join("a.txt"), "x").unwrap();
        commit_all(dir.path(), "root");
        // Detach and delete the branch ref: only raw history remains -> HEAD
        // fallback. (Ref deletion via git, not fs, so packed-refs is honored.)
        for args in [
            ["checkout", "--detach", "HEAD"],
            ["branch", "-D", "main"],
        ] {
            let out = SysCommand::new("git").args(args).current_dir(&dir).output().unwrap();
            assert!(out.status.success(), "git {:?} failed", args);
        }
        let resolved = resolve_base_ref(None, dir.path()).unwrap();
        assert_eq!(resolved, "HEAD");
    }

    #[test]
    fn explicit_missing_ref_is_typed_error() {
        let dir = tempfile::tempdir().unwrap();
        git_init(dir.path());
        std::fs::write(dir.path().join("a.txt"), "x").unwrap();
        commit_all(dir.path(), "root");
        let err = resolve_base_ref(Some("origin/nope"), dir.path()).unwrap_err();
        assert_eq!(
            err,
            BaseRefError::ConfiguredRefMissing {
                ref_: "origin/nope".into()
            }
        );
    }

    #[test]
    fn explicit_configured_ref_wins_when_it_resolves() {
        let dir = tempfile::tempdir().unwrap();
        git_init(dir.path());
        std::fs::write(dir.path().join("a.txt"), "x").unwrap();
        commit_all(dir.path(), "root");
        let resolved = resolve_base_ref(Some("main"), dir.path()).unwrap();
        assert_eq!(resolved, "main");
    }

    #[test]
    fn empty_repo_is_typed_no_base_error() {
        let dir = tempfile::tempdir().unwrap();
        git_init(dir.path());
        let err = resolve_base_ref(None, dir.path()).unwrap_err();
        assert!(matches!(err, BaseRefError::NoBaseResolvable { .. }));
    }

    #[test]
    fn local_main_branch_beats_head_fallback_when_origin_missing() {
        let dir = tempfile::tempdir().unwrap();
        git_init(dir.path());
        std::fs::write(dir.path().join("a.txt"), "x").unwrap();
        commit_all(dir.path(), "root");
        std::fs::write(dir.path().join("b.txt"), "y").unwrap();
        commit_all(dir.path(), "second");
        let resolved = resolve_base_ref(None, dir.path()).unwrap();
        assert_eq!(resolved, "main");
    }
}
