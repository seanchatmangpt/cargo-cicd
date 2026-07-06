use crate::adapters::{GitStatusAdapter, TargetScannerAdapter, ToolchainDetector};
use crate::autonomic::policies::{
    run_all_policies, EvidenceState, GitState, PolicyVerdict, WorkspaceInfo,
};
use crate::evidence::ProcessEvent;
use crate::legacy_nouns::evidence_helpers::{finish_evidence, init_evidence};
use crate::ui::badge::{self, Verdict};
use crate::ui::theme::{self, Role};
use crate::ui::{panel, symbols};
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

/// Render one diagnostic check row as `<badge> <label>`. The verdict tag carries
/// the semantic color (auto-dropped off-TTY) while the `label` stays a
/// contiguous plain-text token — the public-boundary tests grep these labels
/// (e.g. "Cargo.toml"), so the badge is placed before, never inside, the label.
fn check_row(tag: &str, label: &str) -> String {
    format!(
        "{} {}",
        badge::tag(Verdict::from_tag(tag)),
        theme::paint(label, Role::Value)
    )
}

pub struct WorkspaceDoctorVerb;
impl VerbCommand for WorkspaceDoctorVerb {
    fn name(&self) -> &'static str {
        "doctor"
    }
    fn about(&self) -> &'static str {
        "Diagnose workspace health"
    }
    fn build_command(&self) -> clap::Command {
        clap::Command::new(self.name()).about(self.about()).arg(
            clap::Arg::new("json")
                .long("json")
                .action(clap::ArgAction::SetTrue)
                .help("Output machine-readable JSON"),
        )
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let json_mode = std::env::args().any(|a| a == "--json");
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("workspace:doctor");

        if !json_mode {
            println!("{}", panel::header("workspace doctor"));
        }

        let has_cargo = std::path::Path::new("Cargo.toml").exists();
        if !json_mode {
            println!(
                "{}",
                check_row(if has_cargo { "OK" } else { "FAIL" }, "Cargo.toml")
            );
        }

        let toolchain = ToolchainDetector::active_toolchain();
        if !json_mode {
            println!("{}", check_row("OK", &format!("toolchain: {}", toolchain)));
        }

        let has_toolchain_file = std::path::Path::new("rust-toolchain.toml").exists()
            || std::path::Path::new("rust-toolchain").exists();
        if !json_mode {
            println!(
                "{}",
                check_row(
                    if has_toolchain_file { "OK" } else { "WARN" },
                    "rust-toolchain file"
                )
            );
        }

        let has_git = std::path::Path::new(".git").exists();
        if !json_mode {
            println!(
                "{}",
                check_row(if has_git { "OK" } else { "FAIL" }, "git repository")
            );
        }

        let has_cicd = std::path::Path::new("cicd.toml").exists();
        if !json_mode {
            println!(
                "{}",
                check_row(
                    if has_cicd { "OK" } else { "WARN" },
                    "cicd.toml (run 'cargo cicd publish' to generate)"
                )
            );
        }

        // ── autonomic policy checks ──────────────────────────────────────────
        let (target_size_bytes, target_scan_errors) =
            TargetScannerAdapter::total_size_bytes_with_errors("target");
        let target_gb = target_size_bytes as f64 / 1_073_741_824.0;
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

        if !json_mode && target_scan_errors > 0 {
            println!(
                "{}",
                check_row(
                    "WARN",
                    &format!(
                        "target/ size may be undercounted: {} file(s) or folder(s) could not be read while scanning",
                        target_scan_errors
                    )
                )
            );
        }

        if !json_mode && !results.is_empty() {
            println!();
            println!("{}", panel::header("autonomic policy results"));
            for r in &results {
                let tag = match r.verdict {
                    PolicyVerdict::Pass => "PASS",
                    PolicyVerdict::Warn => "WARN",
                    PolicyVerdict::Suggest => "SUGGEST",
                };
                if r.recommendation.is_empty() {
                    println!("{}", check_row(tag, &r.name));
                } else {
                    println!(
                        "{} {}",
                        check_row(tag, &r.name),
                        theme::paint(
                            &format!("{} {}", symbols::arrow(), r.recommendation),
                            Role::Muted
                        )
                    );
                }
            }
        }

        // ── python/shell authority logic check ──────────────────────────────────
        let mut has_authority_scripts = false;
        let mut authority_files = Vec::new();
        let do_fraud_scan = std::env::var("DoctorChecksPythonShellAuthority").as_deref() == Ok("1");

        if do_fraud_scan {
            for entry in walkdir::WalkDir::new(".")
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    name != "target" && name != ".git" && name != "node_modules"
                })
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "py" || ext == "sh" {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            let has_conditionals = content.contains("if [")
                                || content.contains("if test")
                                || content.contains("elif ")
                                || content.contains("case ")
                                || (content.contains("if ") && content.contains(":"))
                                || content.contains("def ")
                                || content.contains("function ");
                            let has_cargo = content.contains("cargo ");
                            if has_conditionals && has_cargo {
                                authority_files.push(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }

            if !authority_files.is_empty() {
                has_authority_scripts = true;
                if !json_mode {
                    println!(
                        "{}",
                        check_row(
                            "FAIL",
                            "Python/shell scripts acting as authority logic (fraud scan)"
                        )
                    );
                    for f in &authority_files {
                        println!("    {} {}", theme::paint("->", Role::Muted), f);
                    }
                }
            } else {
                if !json_mode {
                    println!(
                        "{}",
                        check_row("OK", "No Python/shell authority logic detected")
                    );
                }
            }
        }

        let verdict_str = if !has_cargo || !has_git || has_authority_scripts {
            "FAIL"
        } else {
            "PASS"
        };

        if json_mode {
            let mut checks = vec![];
            checks.push(serde_json::json!({ "name": "Cargo.toml", "status": if has_cargo { "OK" } else { "FAIL" } }));
            checks.push(
                serde_json::json!({ "name": "toolchain", "status": "OK", "value": toolchain }),
            );
            checks.push(serde_json::json!({ "name": "rust-toolchain file", "status": if has_toolchain_file { "OK" } else { "WARN" } }));
            checks.push(serde_json::json!({ "name": "git repository", "status": if has_git { "OK" } else { "FAIL" } }));
            checks.push(serde_json::json!({ "name": "cicd.toml", "status": if has_cicd { "OK" } else { "WARN" } }));
            if target_scan_errors > 0 {
                checks.push(serde_json::json!({
                    "name": "target/ scan completeness",
                    "status": "WARN",
                    "value": format!("{} file(s) or folder(s) could not be read while scanning", target_scan_errors)
                }));
            }
            if do_fraud_scan {
                checks.push(serde_json::json!({ "name": "Python/shell authority fraud scan", "status": if has_authority_scripts { "FAIL" } else { "OK" } }));
            }

            let mut policies_json = vec![];
            for r in &results {
                let tag = match r.verdict {
                    PolicyVerdict::Pass => "PASS",
                    PolicyVerdict::Warn => "WARN",
                    PolicyVerdict::Suggest => "SUGGEST",
                };
                policies_json.push(serde_json::json!({
                    "name": r.name,
                    "verdict": tag,
                    "recommendation": r.recommendation,
                }));
            }
            let out = serde_json::json!({
                "verdict": verdict_str,
                "checks": checks,
                "policies": policies_json
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        } else {
            println!();
            if verdict_str == "FAIL" {
                println!(
                    "{} {}",
                    badge::tag(Verdict::Fail),
                    theme::paint("workspace has critical issues", Role::Danger)
                );
            } else {
                println!(
                    "{} {}",
                    badge::tag(Verdict::Pass),
                    theme::paint("workspace is healthy", Role::Success)
                );
            }
        }

        finish_evidence(
            start_evt,
            t0,
            case_id,
            verdict_str,
            "workspace:doctor",
            &evidence_dir,
        );
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
