#![allow(deprecated)]
use crate::adapters::{GitStatusAdapter, TargetScannerAdapter, ToolchainDetector};
use crate::autonomic::policy_engine;
use crate::engine::EngineState;
use crate::legacy_nouns::evidence_helpers::{finish_evidence, init_evidence};
use crate::ui::badge::{self, Verdict};
use crate::ui::theme::{self, Role};
use crate::ui::{chart, panel};
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

#[deprecated(note = "Use crate::nouns::status::cmd_show / cmd_audit directly (D-T1)")]
pub struct StatusNoun;
impl StatusNoun {
    pub fn new() -> Self {
        Self
    }
}
impl Default for StatusNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for StatusNoun {
    fn name(&self) -> &'static str {
        "status"
    }
    fn about(&self) -> &'static str {
        "Show workspace CI/CD status"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(StatusShowVerb), Box::new(StatusAuditVerb)]
    }
}

impl StatusNoun {
    #[deprecated(note = "Use crate::nouns::status::cmd_show directly (D-T1)")]
    pub fn run_direct() -> anyhow::Result<()> {
        StatusShowVerb.execute()
    }
}

pub struct StatusShowVerb;

impl StatusShowVerb {
    fn execute(&self) -> anyhow::Result<()> {
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("status:show");

        println!("{}", panel::header("cargo-cicd workspace status"));

        let toolchain = ToolchainDetector::active_toolchain();
        let max_gb = 20.0_f64;
        let target_gb = TargetScannerAdapter::total_size_gb("target");
        let verdict_str = TargetScannerAdapter::verdict(target_gb, max_gb);
        let git = GitStatusAdapter::query().unwrap_or_default();
        let dirty = !git.dirty_files.is_empty() || !git.untracked_files.is_empty();
        let dirty_word = if dirty { "dirty" } else { "clean" };

        // Owned, styled value cells. Required literal tokens (the gauge readout's
        // "GB", branch, counts) survive plain mode because color auto-disables
        // off-TTY and `paint` wraps the cell without splitting its text.
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

        // Build real engine state from adapters.
        let engine = EngineState::from_workspace();

        // Show key engine state dimensions.
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

        // Run autonomic policies and display suggestions.
        let suggestions = policy_engine::run_suggestions(&engine);
        if !suggestions.is_empty() {
            println!();
            println!("policy suggestions:");
            for s in &suggestions {
                println!("  → {}", s);
            }
        }

        #[cfg(feature = "lsp")]
        {
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

        Ok(())
    }
}

impl VerbCommand for StatusShowVerb {
    fn name(&self) -> &'static str {
        "show"
    }
    fn about(&self) -> &'static str {
        "Show full CI/CD status"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        self.execute()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
    }
}

// ── audit verb ────────────────────────────────────────────────────────────────

/// `cargo cicd status audit` — shell out to wpm to adjudicate the current
/// evidence OCEL 2.0 file.  Emits an `evidence:audit` event (with oracle provenance)
/// back into the log, then fails if the oracle refuses.
pub struct StatusAuditVerb;

impl StatusAuditVerb {
    fn execute(&self) -> anyhow::Result<()> {
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

        // Detect wpm oracle.
        let wpm = crate::integrations::Wasm4pmShell::detect();
        let Some(wpm_shell) = wpm else {
            println!("{} wpm oracle not found", badge::tag(Verdict::Blocked));
            return Ok(());
        };

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

        let result = wpm_shell
            .audit(ocel.to_str().unwrap_or(""))
            .unwrap_or_else(|e| crate::integrations::WpmResult {
                command: "wpm audit".to_string(),
                success: false,
                stdout: String::new(),
                stderr: e.to_string(),
                verdict: crate::integrations::WpmVerdict::Fail,
            });

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
        let rows_ref: Vec<(&str, &str)> =
            result_rows.iter().map(|(k, v)| (*k, v.as_str())).collect();
        println!("{}", panel::kv(&rows_ref));

        let oracle_verdict = if result.success { "ACCEPT" } else { "REFUSE" };
        println!(
            "{} {}",
            theme::paint("verdict:", Role::Label),
            badge::tag(Verdict::from_tag(oracle_verdict))
        );

        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        // 1. Emit status:audit — the declared required_stage event.
        let (mut sa_start, sa_t0) = crate::evidence::ProcessEvent::started("status:audit");
        sa_start.case_id = Some(case_id.clone());
        let mut sa_complete =
            crate::evidence::ProcessEvent::completed("status:audit", sa_t0, oracle_verdict);
        sa_complete.case_id = Some(case_id.clone());

        // 2. Emit evidence:audit — adjudicated event with oracle provenance.
        let mut ea_evt = crate::evidence::ProcessEvent::new_adjudicated(
            "evidence:audit",
            oracle_verdict,
            wpm_shell.binary_path(),
        );
        ea_evt.case_id = Some(case_id.clone());

        // 3. Emit receipt:write — only if oracle accepted.
        let mut events_to_append: Vec<crate::evidence::ProcessEvent> =
            vec![sa_start, sa_complete, ea_evt];

        if result.success {
            let mut rw_evt = crate::evidence::ProcessEvent::new("receipt:write", "COMPLETE");
            rw_evt.case_id = Some(case_id.clone());
            events_to_append.push(rw_evt);
        }

        // Append all three/four events into the main evidence log so they are
        // included in the next evidence re-build.
        if let Err(e) = crate::evidence::append_events(&events_to_append, &evidence_dir) {
            eprintln!("warning: audit evidence emission failed: {}", e);
        }

        if !result.success {
            anyhow::bail!("wasm4pm REFUSED evidence");
        }
        Ok(())
    }
}

impl VerbCommand for StatusAuditVerb {
    fn name(&self) -> &'static str {
        "audit"
    }
    fn about(&self) -> &'static str {
        "Adjudicate current evidence OCEL 2.0 file via the wasm4pm oracle"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        self.execute()
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
    }
}
