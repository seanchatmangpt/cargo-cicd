//! Switches between running the workspace's GitHub Actions workflows locally
//! via `act` and dispatching them remotely via `gh workflow run` / `gh run watch`.
use crate::evidence_helpers::{finish_evidence, init_evidence};
use crate::ui::badge::{self, Verdict};
use crate::ui::theme::{self, Role};
use crate::ui::panel;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CiMode {
    Local,
    Remote,
}

impl CiMode {
    fn from_flag(mode: Option<&str>) -> Self {
        match mode {
            Some("remote") => CiMode::Remote,
            _ => CiMode::Local,
        }
    }
    fn as_str(self) -> &'static str {
        match self {
            CiMode::Local => "local",
            CiMode::Remote => "remote",
        }
    }
}

fn binary_on_path(bin: &str) -> bool {
    std::process::Command::new(bin)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn current_branch() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "main".to_string())
}

fn run_local(workflow: &str, dry_run: bool) -> (&'static str, &'static str) {
    let cmd_line = format!("act --workflows .github/workflows/{}", workflow);
    println!("{} {}", theme::paint("command:", Role::Label), cmd_line);

    if dry_run {
        return ("dry_run", "WARN");
    }
    if !binary_on_path("act") {
        println!(
            "{} `act` not found on PATH — install act to run workflows locally",
            badge::tag(Verdict::Blocked)
        );
        return ("blocked", "WARN:oracle_unavailable");
    }

    let status = std::process::Command::new("act")
        .args(["--workflows", &format!(".github/workflows/{}", workflow)])
        .status();
    match status {
        Ok(s) if s.success() => ("pass", "PASS"),
        Ok(_) => ("fail", "FAIL"),
        Err(e) => {
            println!("{} failed to spawn `act`: {}", badge::tag(Verdict::Blocked), e);
            ("blocked", "WARN:oracle_unavailable")
        }
    }
}

fn run_remote(workflow: &str, dry_run: bool, watch: bool) -> (&'static str, &'static str) {
    let branch = current_branch();
    let dispatch_cmd = format!("gh workflow run {} --ref {}", workflow, branch);
    println!("{} {}", theme::paint("command:", Role::Label), dispatch_cmd);
    if watch {
        println!("{} gh run watch", theme::paint("follow-up:", Role::Label));
    }

    if dry_run {
        return ("dry_run", "WARN");
    }
    if !binary_on_path("gh") {
        println!(
            "{} `gh` not found on PATH — install the GitHub CLI to dispatch remote runs",
            badge::tag(Verdict::Blocked)
        );
        return ("blocked", "WARN:oracle_unavailable");
    }

    let dispatch = std::process::Command::new("gh")
        .args(["workflow", "run", workflow, "--ref", &branch])
        .status();
    let dispatched = matches!(dispatch, Ok(s) if s.success());
    if !dispatched {
        return ("fail", "FAIL");
    }
    if watch {
        let watched = std::process::Command::new("gh").args(["run", "watch"]).status();
        return match watched {
            Ok(s) if s.success() => ("pass", "PASS"),
            Ok(_) => ("fail", "FAIL"),
            Err(_) => ("pass", "PASS"),
        };
    }
    ("pass", "PASS")
}

fn run_ci(mode: CiMode, workflow: &str, dry_run: bool, watch: bool) -> anyhow::Result<()> {
    let (evidence_dir, case_id, start_evt, t0) = init_evidence("ci:run");

    println!("{}", panel::header("cargo-cicd ci run"));
    let mode_v = theme::paint(mode.as_str(), Role::Value);
    let workflow_v = theme::paint(workflow, Role::Value);
    let dry_run_v = theme::paint(if dry_run { "yes" } else { "no" }, Role::Value);
    println!(
        "{}",
        panel::kv(&[
            ("mode", mode_v.as_str()),
            ("workflow", workflow_v.as_str()),
            ("dry-run", dry_run_v.as_str()),
        ])
    );

    let (verdict_tag, ev_verdict) = match mode {
        CiMode::Local => run_local(workflow, dry_run),
        CiMode::Remote => run_remote(workflow, dry_run, watch),
    };

    println!(
        "{} {}",
        theme::paint("verdict:", Role::Label),
        badge::tag(Verdict::from_tag(verdict_tag))
    );

    finish_evidence(start_evt, t0, case_id, ev_verdict, "ci:run", &evidence_dir);

    if ev_verdict == "FAIL" {
        anyhow::bail!("ci run failed");
    }
    Ok(())
}

/// Run a workflow locally via act or remotely via GitHub Actions.
#[verb("run")]
pub fn cmd_run(mode: Option<String>, workflow: Option<String>, watch: bool, dry_run: bool) -> Result<()> {
    let mode = CiMode::from_flag(mode.as_deref());
    let workflow = workflow.unwrap_or_else(|| "ci.yml".to_string());
    run_ci(mode, &workflow, dry_run, watch)
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}
