use crate::adapters::GitStatusAdapter;
use crate::evidence::ProcessEvent;
use crate::ui::badge::{self, Verdict};
use crate::ui::theme::{self, Role};
use crate::ui::panel;
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct GitNoun;
impl GitNoun {
    pub fn new() -> Self {
        Self
    }
}
impl Default for GitNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for GitNoun {
    fn name(&self) -> &'static str {
        "git"
    }
    fn about(&self) -> &'static str {
        "Git phase management"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![
            Box::new(GitStatusVerb),
            Box::new(GitCloseVerb),
            Box::new(GitDiffVerb),
            Box::new(GitStageVerb),
            Box::new(GitCommitVerb),
            Box::new(GitFetchVerb),
        ]
    }
}

/// Render a single changed-file row as `<marker> <path>`. The `marker` glyph is
/// colored by `role` (color auto-drops off-TTY); the `path` is emitted as a
/// contiguous plain token so the public-boundary tests can grep specific
/// filenames in captured output.
fn file_row(marker: &str, role: Role, path: &str) -> String {
    format!("  {} {}", theme::paint(marker, role), path)
}

pub struct GitStatusVerb;
impl VerbCommand for GitStatusVerb {
    fn name(&self) -> &'static str {
        "status"
    }
    fn about(&self) -> &'static str {
        "Show git repository state"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("git:status");
        start_evt.case_id = Some(case_id.clone());

        let status = GitStatusAdapter::query()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;

        let clean = status.dirty_files.is_empty() && status.untracked_files.is_empty();

        println!("{}", panel::header("git status"));

        // Aligned key/value summary. Counts and the branch name stay contiguous
        // plain tokens off-TTY; the working-tree state carries a verdict badge.
        let branch_v = theme::paint(&status.branch, Role::Value);
        let staged_n = status.staged_files.len().to_string();
        let dirty_n = status.dirty_files.len().to_string();
        let untracked_n = status.untracked_files.len().to_string();
        let ahead_n = status.ahead.to_string();
        let behind_n = status.behind.to_string();
        let tree_v = badge::tag(Verdict::from_tag(if clean { "clean" } else { "dirty" }));
        println!(
            "{}",
            panel::kv(&[
                ("branch", branch_v.as_str()),
                ("tree", tree_v.as_str()),
                ("staged", staged_n.as_str()),
                ("dirty", dirty_n.as_str()),
                ("untracked", untracked_n.as_str()),
                ("ahead", ahead_n.as_str()),
                ("behind", behind_n.as_str()),
            ])
        );
        println!();
        if !status.dirty_files.is_empty() {
            println!("{}", theme::paint("dirty files:", Role::Label));
            for f in &status.dirty_files {
                println!("{}", file_row("M", Role::Warning, f));
            }
        }
        if !status.untracked_files.is_empty() {
            println!("{}", theme::paint("untracked:", Role::Label));
            for f in &status.untracked_files {
                println!("{}", file_row("?", Role::Info, f));
            }
        }
        let next = if clean {
            "tree is clean — ready to push"
        } else {
            "recommendation: run 'cargo cicd git close' to stage and commit"
        };
        println!(
            "{} {}",
            theme::paint("next:", Role::Label),
            theme::paint(next, if clean { Role::Success } else { Role::Warning })
        );
        let ev_verdict = if clean { "PASS" } else { "WARN" };
        let mut complete_evt = ProcessEvent::completed("git:status", t0, ev_verdict);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

pub struct GitCloseVerb;
impl VerbCommand for GitCloseVerb {
    fn name(&self) -> &'static str {
        "close"
    }
    fn about(&self) -> &'static str {
        "Enforce phase closure: stage outputs, commit, verify clean"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let status = GitStatusAdapter::query()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;
        println!("{}", panel::header("git phase closure"));
        let dirty_before = status.dirty_files.len() + status.untracked_files.len();
        if status.dirty_files.is_empty() && status.untracked_files.is_empty() {
            println!(
                "{} {}",
                badge::tag(Verdict::Pass),
                theme::paint("tree is clean — phase already closed", Role::Success)
            );
            let evidence_dir = crate::evidence::evidence_dir();
            let case_id = crate::session::read_or_create_session_id(&evidence_dir);
            let (mut start_evt, t0) = ProcessEvent::started("git:close");
            start_evt.case_id = Some(case_id.clone());
            let mut complete_evt = ProcessEvent::completed("git:close", t0, "PASS");
            complete_evt.case_id = Some(case_id);
            if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
                eprintln!("warning: evidence emission failed: {}", e);
            }
            return Ok(());
        }
        println!(
            "{}",
            panel::kv(&[
                ("dirty files", status.dirty_files.len().to_string().as_str()),
                ("untracked", status.untracked_files.len().to_string().as_str()),
            ])
        );
        println!();
        println!(
            "{}",
            theme::paint("phase closure requires a clean tree.", Role::Warning)
        );
        println!("stage and commit your changes before closing the phase.");
        println!();
        println!(
            "{} {}",
            badge::tag(Verdict::Blocked),
            theme::paint(
                "refusing to hide unrelated dirty files — no silent batch commit.",
                Role::Danger
            )
        );
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("git:close");
        start_evt.case_id = Some(case_id.clone());
        let mut complete_evt = ProcessEvent::completed("git:close", t0, "FAIL");
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        let _ = dirty_before;
        Err(clap_noun_verb::error::NounVerbError::execution_error(
            "phase closure refused: tree is dirty. Stage and commit manually, then re-run.",
        ))
    }
}

pub struct GitDiffVerb;
impl VerbCommand for GitDiffVerb {
    fn name(&self) -> &'static str {
        "diff"
    }
    fn about(&self) -> &'static str {
        "Show unstaged and staged changes in the working tree"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("git:diff");
        start_evt.case_id = Some(case_id.clone());

        let unstaged = std::process::Command::new("git")
            .args(["diff", "--stat"])
            .output()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;
        let staged = std::process::Command::new("git")
            .args(["diff", "--cached", "--stat"])
            .output()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;

        let unstaged_out = String::from_utf8_lossy(&unstaged.stdout);
        let staged_out = String::from_utf8_lossy(&staged.stdout);

        println!("unstaged:");
        if unstaged_out.trim().is_empty() {
            println!("  (none)");
        } else {
            for line in unstaged_out.lines() {
                println!("  {}", line);
            }
        }
        println!("staged:");
        if staged_out.trim().is_empty() {
            println!("  (none)");
        } else {
            for line in staged_out.lines() {
                println!("  {}", line);
            }
        }

        let verdict_str = if unstaged_out.trim().is_empty() && staged_out.trim().is_empty() {
            "PASS"
        } else {
            "WARN"
        };

        let mut complete_evt = ProcessEvent::completed("git:diff", t0, verdict_str);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

pub struct GitStageVerb;
impl VerbCommand for GitStageVerb {
    fn name(&self) -> &'static str {
        "stage"
    }
    fn about(&self) -> &'static str {
        "Stage all modified tracked files (git add -u)"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("git:stage");
        start_evt.case_id = Some(case_id.clone());

        let add_result = std::process::Command::new("git")
            .args(["add", "-u"])
            .output()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;

        let verdict_str = if add_result.status.success() {
            // Count staged files from git status after staging
            let status_out = std::process::Command::new("git")
                .args(["status", "--short"])
                .output()
                .ok();
            let staged_count = status_out
                .as_ref()
                .map(|o| {
                    String::from_utf8_lossy(&o.stdout)
                        .lines()
                        .filter(|l| l.starts_with('M') || l.starts_with('D') || l.starts_with('R'))
                        .count()
                })
                .unwrap_or(0);
            println!("staged {} modified files", staged_count);
            "PASS"
        } else {
            let err = String::from_utf8_lossy(&add_result.stderr);
            eprintln!("git add -u failed: {}", err);
            "FAIL"
        };

        let mut complete_evt = ProcessEvent::completed("git:stage", t0, verdict_str);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }

        if verdict_str == "FAIL" {
            return Err(clap_noun_verb::error::NounVerbError::execution_error(
                "git add -u failed",
            ));
        }
        Ok(())
    }
}

pub struct GitCommitVerb;
impl VerbCommand for GitCommitVerb {
    fn name(&self) -> &'static str {
        "commit"
    }
    fn about(&self) -> &'static str {
        "Create a commit with auto-generated message from cicd.toml state"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("git:commit");
        start_evt.case_id = Some(case_id.clone());

        // Check only tracked-file modifications (ignore untracked dirs like target/).
        let status_out = std::process::Command::new("git")
            .args(["status", "--short", "-uno"])
            .output()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;
        let status_str = String::from_utf8_lossy(&status_out.stdout);

        let verdict_str;
        if status_str.trim().is_empty() {
            println!("tree is already clean — nothing to commit");
            verdict_str = "WARN";
            let mut complete_evt = ProcessEvent::completed("git:commit", t0, verdict_str);
            complete_evt.case_id = Some(case_id);
            if let Err(e) =
                crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir)
            {
                eprintln!("warning: evidence emission failed: {}", e);
            }
            return Ok(());
        }

        // Stage all tracked modifications first
        let _ = std::process::Command::new("git")
            .args(["add", "-u"])
            .output();

        let commit_result = std::process::Command::new("git")
            .args([
                "commit",
                "-m",
                "chore(cicd): auto-commit from cargo cicd git commit",
            ])
            .output()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;

        if commit_result.status.success() {
            // Extract the short SHA from git rev-parse
            let sha_out = std::process::Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()
                .ok();
            let sha = sha_out
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            println!("committed: {}", sha);
            verdict_str = "PASS";
        } else {
            let err = String::from_utf8_lossy(&commit_result.stderr);
            eprintln!("git commit failed: {}", err);
            verdict_str = "FAIL";
        }

        let mut complete_evt = ProcessEvent::completed("git:commit", t0, verdict_str);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }

        if verdict_str == "FAIL" {
            return Err(clap_noun_verb::error::NounVerbError::execution_error(
                "git commit failed",
            ));
        }
        Ok(())
    }
}

pub struct GitFetchVerb;
impl VerbCommand for GitFetchVerb {
    fn name(&self) -> &'static str {
        "fetch"
    }
    fn about(&self) -> &'static str {
        "Fetch from remote origin"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("git:fetch");
        start_evt.case_id = Some(case_id.clone());

        let fetch_result = std::process::Command::new("git")
            .args(["fetch", "origin"])
            .output()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))?;

        let verdict_str = if fetch_result.status.success() {
            println!("fetch complete");
            "PASS"
        } else {
            let err = String::from_utf8_lossy(&fetch_result.stderr);
            eprintln!("git fetch origin failed: {}", err);
            "FAIL"
        };

        let mut complete_evt = ProcessEvent::completed("git:fetch", t0, verdict_str);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }

        if verdict_str == "FAIL" {
            return Err(clap_noun_verb::error::NounVerbError::execution_error(
                "git fetch origin failed",
            ));
        }
        Ok(())
    }
}
