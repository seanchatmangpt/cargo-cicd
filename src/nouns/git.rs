//! src/nouns/git.rs — cargo cicd git verbs (inlined from the retired legacy layer; see CCICD-106)

use crate::adapters::GitStatusAdapter;
use crate::evidence::ProcessEvent;
use crate::ui::badge::{self, Verdict};
use crate::ui::panel;
use crate::ui::theme::{self, Role};

/// Render a single changed-file row as `<marker> <path>`. The `marker` glyph is
/// colored by `role` (color auto-drops off-TTY); the `path` is emitted as a
/// contiguous plain token so the public-boundary tests can grep specific
/// filenames in captured output.
fn file_row(marker: &str, role: Role, path: &str) -> String {
    format!("  {} {}", theme::paint(marker, role), path)
}

pub struct GitStatusVerb;
impl GitStatusVerb {
    pub fn execute(&self) -> clap_noun_verb::error::Result<()> {
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
impl GitCloseVerb {
    pub fn execute(&self) -> clap_noun_verb::error::Result<()> {
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
            if let Err(e) =
                crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir)
            {
                eprintln!("warning: evidence emission failed: {}", e);
            }
            return Ok(());
        }
        println!(
            "{}",
            panel::kv(&[
                ("dirty files", status.dirty_files.len().to_string().as_str()),
                (
                    "untracked",
                    status.untracked_files.len().to_string().as_str()
                ),
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
impl GitDiffVerb {
    pub fn execute(&self) -> clap_noun_verb::error::Result<()> {
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
impl GitStageVerb {
    pub fn execute(&self) -> clap_noun_verb::error::Result<()> {
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
impl GitCommitVerb {
    pub fn execute(&self) -> clap_noun_verb::error::Result<()> {
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
impl GitFetchVerb {
    pub fn execute(&self) -> clap_noun_verb::error::Result<()> {
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

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[verb("status")]
pub fn cmd_status() -> Result<()> {
    GitStatusVerb.execute()
}

#[verb("close")]
pub fn cmd_close() -> Result<()> {
    GitCloseVerb.execute()
}

#[verb("diff")]
pub fn cmd_diff() -> Result<()> {
    GitDiffVerb.execute()
}

#[verb("stage")]
pub fn cmd_stage() -> Result<()> {
    GitStageVerb.execute()
}

#[verb("commit")]
pub fn cmd_commit() -> Result<()> {
    GitCommitVerb.execute()
}

#[verb("fetch")]
pub fn cmd_fetch() -> Result<()> {
    GitFetchVerb.execute()
}
