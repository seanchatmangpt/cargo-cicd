use crate::adapters::{GitStatusAdapter, TargetScannerAdapter, ToolchainDetector};
use crate::autonomic::policies::{
    run_all_policies, EvidenceState, GitState, PolicyVerdict, WorkspaceInfo,
};
use crate::evidence::ProcessEvent;
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct WorkspaceNoun;
impl WorkspaceNoun {
    pub fn new() -> Self {
        Self
    }
    pub fn run_doctor() -> anyhow::Result<()> {
        WorkspaceDoctorVerb
            .run(&VerbArgs::new(clap::ArgMatches::default()))
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}
impl Default for WorkspaceNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for WorkspaceNoun {
    fn name(&self) -> &'static str {
        "workspace"
    }
    fn about(&self) -> &'static str {
        "Workspace diagnostics"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![
            Box::new(WorkspaceDoctorVerb),
            Box::new(WorkspaceValidateVerb),
            Box::new(WorkspaceSyncVerb),
            Box::new(WorkspaceListVerb),
        ]
    }
}

pub struct WorkspaceDoctorVerb;
impl VerbCommand for WorkspaceDoctorVerb {
    fn name(&self) -> &'static str {
        "doctor"
    }
    fn about(&self) -> &'static str {
        "Diagnose workspace health"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("workspace:doctor");
        start_evt.case_id = Some(case_id.clone());

        println!("workspace doctor");
        println!("================");

        let has_cargo = std::path::Path::new("Cargo.toml").exists();
        println!("[{}] Cargo.toml", if has_cargo { "OK" } else { "FAIL" });

        let toolchain = ToolchainDetector::active_toolchain();
        println!("[OK] toolchain: {}", toolchain);

        let has_toolchain_file = std::path::Path::new("rust-toolchain.toml").exists()
            || std::path::Path::new("rust-toolchain").exists();
        println!(
            "[{}] rust-toolchain file",
            if has_toolchain_file { "OK" } else { "WARN" }
        );

        let has_git = std::path::Path::new(".git").exists();
        println!("[{}] git repository", if has_git { "OK" } else { "FAIL" });

        let has_cicd = std::path::Path::new("cicd.toml").exists();
        println!(
            "[{}] cicd.toml (run 'cargo cicd publish' to generate)",
            if has_cicd { "OK" } else { "WARN" }
        );

        // ── autonomic policy checks ──────────────────────────────────────────
        let target_gb = TargetScannerAdapter::total_size_gb("target");
        let pinned_toolchain = read_pinned_toolchain();
        let git_dirty = GitStatusAdapter::query()
            .map(|r| r.dirty_files.len())
            .unwrap_or(0);

        let workspace_info = WorkspaceInfo {
            target_gb,
            max_gb: 20.0,
            active_toolchain: toolchain.clone(),
            pinned_toolchain,
            changed_trybuild_fixtures: 0,
        };
        let git_state = GitState {
            dirty_count: git_dirty,
            commits_behind: None,
        };
        let evidence_state = EvidenceState {
            changed_file_count: 0,
            evidence_fresh: true,
            receipt_exists: false,
            receipt_stale: false,
        };

        let results = run_all_policies(&workspace_info, &git_state, &evidence_state);

        if !results.is_empty() {
            println!();
            println!("autonomic policy results");
            println!("------------------------");
            for r in &results {
                let tag = match r.verdict {
                    PolicyVerdict::Pass => "PASS",
                    PolicyVerdict::Warn => "WARN",
                    PolicyVerdict::Suggest => "SUGGEST",
                };
                if r.recommendation.is_empty() {
                    println!("[{}] {}", tag, r.name);
                } else {
                    println!("[{}] {}: {}", tag, r.name, r.recommendation);
                }
            }
        }

        println!();
        let verdict_str = if !has_cargo || !has_git {
            println!("FAIL: workspace has critical issues");
            "FAIL"
        } else {
            println!("workspace is healthy");
            "PASS"
        };

        let mut complete_evt = ProcessEvent::completed("workspace:doctor", t0, verdict_str);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

/// Read the channel from `rust-toolchain.toml` if it exists.
fn read_pinned_toolchain() -> Option<String> {
    if std::path::Path::new("rust-toolchain.toml").exists() {
        std::fs::read_to_string("rust-toolchain.toml")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.contains("channel"))
                    .and_then(|l| l.split('"').nth(1))
                    .map(|s| s.to_string())
            })
    } else {
        None
    }
}

pub struct WorkspaceValidateVerb;
impl VerbCommand for WorkspaceValidateVerb {
    fn name(&self) -> &'static str {
        "validate"
    }
    fn about(&self) -> &'static str {
        "Validate workspace Cargo.toml structure and declared members"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("workspace:validate");
        start_evt.case_id = Some(case_id.clone());

        println!("workspace validate");
        println!("==================");

        let cargo_toml_path = std::path::Path::new("Cargo.toml");
        let cargo_toml_exists = cargo_toml_path.exists();
        println!(
            "[{}] Cargo.toml exists",
            if cargo_toml_exists { "PASS" } else { "FAIL" }
        );

        let mut overall = if cargo_toml_exists { "PASS" } else { "FAIL" };

        if cargo_toml_exists {
            let content = std::fs::read_to_string(cargo_toml_path).unwrap_or_default();

            let has_workspace_section = content.contains("[workspace]");
            println!(
                "[{}] [workspace] section present",
                if has_workspace_section {
                    "PASS"
                } else {
                    "FAIL"
                }
            );
            if !has_workspace_section {
                overall = "FAIL";
            }

            // Parse members and check each exists on disk.
            let members = parse_workspace_members(&content);
            if members.is_empty() {
                println!("[WARN] no workspace members declared (single-crate workspace?)");
            } else {
                for member in &members {
                    let member_path = std::path::Path::new(member);
                    let exists = member_path.exists();
                    println!(
                        "[{}] member {} exists on disk",
                        if exists { "PASS" } else { "FAIL" },
                        member
                    );
                    if !exists {
                        overall = "FAIL";
                    }
                }
            }
        }

        println!();
        println!("validate verdict: {}", overall);

        let mut complete_evt = ProcessEvent::completed("workspace:validate", t0, overall);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

pub struct WorkspaceSyncVerb;
impl VerbCommand for WorkspaceSyncVerb {
    fn name(&self) -> &'static str {
        "sync"
    }
    fn about(&self) -> &'static str {
        "Sync workspace via ggen if ggen.toml is present"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("workspace:sync");
        start_evt.case_id = Some(case_id.clone());

        let verdict = if std::path::Path::new("ggen.toml").exists() {
            match std::process::Command::new("ggen").arg("sync").output() {
                Ok(out) if out.status.success() => {
                    println!("{}", String::from_utf8_lossy(&out.stdout));
                    "PASS"
                }
                Ok(out) => {
                    eprintln!("{}", String::from_utf8_lossy(&out.stderr));
                    "FAIL"
                }
                Err(_) => {
                    println!("ggen not found; skipping sync");
                    "WARN:ggen_unavailable"
                }
            }
        } else {
            println!("ggen.toml not found; no sync needed");
            "PASS"
        };

        let mut complete_evt = ProcessEvent::completed("workspace:sync", t0, verdict);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

pub struct WorkspaceListVerb;
impl VerbCommand for WorkspaceListVerb {
    fn name(&self) -> &'static str {
        "list"
    }
    fn about(&self) -> &'static str {
        "List all workspace member crates"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("workspace:list");
        start_evt.case_id = Some(case_id.clone());

        println!("workspace members");
        println!("=================");

        let content = std::fs::read_to_string("Cargo.toml").unwrap_or_default();
        let members = parse_workspace_members(&content);
        if members.is_empty() {
            println!("  (no workspace members declared — single-crate workspace)");
        } else {
            for member in &members {
                println!("  {}", member);
            }
            println!();
            println!("total: {} member(s)", members.len());
        }

        let mut complete_evt = ProcessEvent::completed("workspace:list", t0, "PASS");
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

/// Parse the `members = [...]` array from a Cargo.toml string.
fn parse_workspace_members(content: &str) -> Vec<String> {
    let mut in_workspace = false;
    let mut in_members = false;
    let mut members = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace]" {
            in_workspace = true;
            continue;
        }
        if in_workspace && trimmed.starts_with('[') && trimmed != "[workspace]" {
            in_workspace = false;
            in_members = false;
        }
        if in_workspace && trimmed.starts_with("members") {
            in_members = true;
        }
        if in_members {
            let mut chars = trimmed.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '"' {
                    let member: String = chars.by_ref().take_while(|&c| c != '"').collect();
                    if !member.is_empty() {
                        members.push(member);
                    }
                }
            }
            if trimmed.contains(']') {
                in_members = false;
            }
        }
    }

    members
}
