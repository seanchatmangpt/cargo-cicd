use crate::adapters::{GitStatusAdapter, TargetScannerAdapter, ToolchainDetector};
use crate::autonomic::policy_engine;
use crate::engine::EngineState;
use crate::evidence_helpers::{finish_evidence, init_evidence};
use crate::ui::badge::{self, Verdict};
use crate::ui::theme::{self, Role};
use crate::ui::{chart, panel};
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

// ── domain helpers ────────────────────────────────────────────────────────────

fn run_show() -> anyhow::Result<()> {
    let (evidence_dir, case_id, start_evt, t0) = init_evidence("status:show");

    println!("{}", panel::header("cargo-cicd workspace status"));

    let toolchain = ToolchainDetector::active_toolchain();
    let max_gb = 20.0_f64;
    let target_gb = TargetScannerAdapter::total_size_gb("target");
    let verdict_str = TargetScannerAdapter::verdict(target_gb, max_gb);
    let git = GitStatusAdapter::query().unwrap_or_default();
    let dirty = !git.dirty_files.is_empty() || !git.untracked_files.is_empty();
    let dirty_word = if dirty { "dirty" } else { "clean" };

    let toolchain_v = theme::paint(&toolchain, Role::Value);
    let target_v = format!(
        "{}  {:.2} GB  {}",
        chart::gauge(target_gb, max_gb, 16),
        target_gb,
        badge::tag(Verdict::from_tag(verdict_str)),
    );
    let branch_v = theme::paint(&git.branch, Role::Value);
    let dirty_n = git.dirty_files.len().to_string();
    let untracked_n = git.untracked_files.len().to_string();
    let git_v = badge::tag(Verdict::from_tag(dirty_word));

    println!(
        "{}",
        panel::kv(&[
            ("toolchain", toolchain_v.as_str()),
            ("target", target_v.as_str()),
            ("branch", branch_v.as_str()),
            ("dirty files", dirty_n.as_str()),
            ("untracked", untracked_n.as_str()),
            ("git", git_v.as_str()),
        ])
    );

    let ev_verdict = if dirty { "WARN" } else { "PASS" };
    finish_evidence(
        start_evt,
        t0,
        case_id,
        ev_verdict,
        "status:show",
        &evidence_dir,
    );

    print_engine_state();
    Ok(())
}

fn print_engine_state() {
    let engine = EngineState::from_workspace();

    println!();
    println!("engine state:");
    if !engine.workspace.root_path.is_empty() {
        println!("  workspace root: {}", engine.workspace.root_path);
    }
    if !engine.workspace.name.is_empty() {
        println!("  workspace name: {}", engine.workspace.name);
    }
    println!("  branch:         {}", engine.git_phase.branch);
    println!("  dirty files:    {}", engine.git_phase.dirty_files.len());
    println!(
        "  target size:    {:.2} GB",
        engine.target.total_size_bytes as f64 / 1_073_741_824.0
    );
    println!("  toolchain:      {}", engine.toolchain.active);

    let suggestions = policy_engine::run_suggestions(&engine);
    if !suggestions.is_empty() {
        println!();
        println!("policy suggestions:");
        for s in &suggestions {
            println!("  → {}", s);
        }
    }

    #[cfg(feature = "lsp")]
    print_lsp_findings();
}

#[cfg(feature = "lsp")]
fn print_lsp_findings() {
    let snapshot =
        cargo_cicd_core::workspace::WorkspaceSnapshot::from_path(std::path::Path::new("."));
    let findings = cargo_cicd_lsp::analyzers::run_all(&snapshot);
    if !findings.is_empty() {
        println!();
        println!("diagnostic findings (from LSP analyzers)");
        for finding in findings {
            println!(
                "[{}] {}: {}",
                finding.severity,
                finding.code.as_str(),
                finding.message
            );
            for repair in &finding.repairs {
                println!("  → {}", repair);
            }
        }
    }
}

fn run_audit() -> anyhow::Result<()> {
    let evidence_dir = crate::evidence::evidence_dir();
    let ocel = evidence_dir.join("events.ocel.json");

    if !ocel.exists() {
        println!(
            "{} no evidence at {}",
            badge::tag(Verdict::Blocked),
            ocel.display()
        );
        return Ok(());
    }

    let wpm = crate::integrations::Wasm4pmShell::detect();
    let Some(wpm_shell) = wpm else {
        println!("{} wpm oracle not found", badge::tag(Verdict::Blocked));
        return Ok(());
    };

    let result = invoke_wpm_audit(&wpm_shell, &ocel);
    emit_audit_events(&wpm_shell, &evidence_dir, &result)?;
    Ok(())
}

fn invoke_wpm_audit(
    wpm_shell: &crate::integrations::Wasm4pmShell,
    ocel: &std::path::Path,
) -> crate::integrations::WpmResult {
    println!("{}", panel::header("wasm4pm evidence audit"));
    let ocel_disp = ocel.display().to_string();
    let wpm_path = wpm_shell.binary_path().to_string();
    println!(
        "{}",
        panel::kv(&[
            ("evidence", theme::paint(&ocel_disp, Role::Value).as_str()),
            ("wpm", theme::paint(&wpm_path, Role::Value).as_str()),
        ])
    );

    wpm_shell
        .audit(ocel.to_str().unwrap_or(""))
        .unwrap_or_else(|e| crate::integrations::WpmResult {
            command: "wpm audit".to_string(),
            success: false,
            stdout: String::new(),
            stderr: e.to_string(),
            verdict: crate::integrations::WpmVerdict::Fail,
        })
}

fn emit_audit_events(
    wpm_shell: &crate::integrations::Wasm4pmShell,
    evidence_dir: &std::path::Path,
    result: &crate::integrations::WpmResult,
) -> anyhow::Result<()> {
    let exit_v = if result.success { "0" } else { "non-zero" };
    let stdout_v = result.stdout.trim().to_string();
    let mut result_rows: Vec<(&str, String)> = vec![
        ("exit", theme::paint(exit_v, Role::Value)),
        ("stdout", stdout_v.clone()),
    ];
    let stderr_trim = result.stderr.trim().to_string();
    if !stderr_trim.is_empty() {
        result_rows.push(("stderr", theme::paint(&stderr_trim, Role::Warning)));
    }
    let rows_ref: Vec<(&str, &str)> = result_rows.iter().map(|(k, v)| (*k, v.as_str())).collect();
    println!("{}", panel::kv(&rows_ref));

    let oracle_verdict = if result.success { "ACCEPT" } else { "REFUSE" };
    println!(
        "{} {}",
        theme::paint("verdict:", Role::Label),
        badge::tag(Verdict::from_tag(oracle_verdict))
    );

    let case_id = crate::session::read_or_create_session_id(evidence_dir);
    let (mut sa_start, sa_t0) = crate::evidence::ProcessEvent::started("status:audit");
    sa_start.case_id = Some(case_id.clone());
    let mut sa_complete =
        crate::evidence::ProcessEvent::completed("status:audit", sa_t0, oracle_verdict);
    sa_complete.case_id = Some(case_id.clone());
    let mut ea_evt = crate::evidence::ProcessEvent::new_adjudicated(
        "evidence:audit",
        oracle_verdict,
        wpm_shell.binary_path(),
    );
    ea_evt.case_id = Some(case_id.clone());

    let mut events_to_append: Vec<crate::evidence::ProcessEvent> =
        vec![sa_start, sa_complete, ea_evt];
    if result.success {
        let mut rw_evt = crate::evidence::ProcessEvent::new("receipt:write", "COMPLETE");
        rw_evt.case_id = Some(case_id.clone());
        events_to_append.push(rw_evt);
    }

    if let Err(e) = crate::evidence::append_events(&events_to_append, evidence_dir) {
        eprintln!("warning: audit evidence emission failed: {}", e);
    }

    if !result.success {
        anyhow::bail!("wasm4pm REFUSED evidence");
    }
    Ok(())
}

// ── verb entry points ─────────────────────────────────────────────────────────

#[verb("show")]
pub fn cmd_show() -> Result<()> {
    run_show().map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}

#[verb("audit")]
pub fn cmd_audit() -> Result<()> {
    run_audit().map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}
