use crate::evidence::ProcessEvent;
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct LspNoun;
impl LspNoun {
    pub fn new() -> Self {
        Self
    }
}
impl Default for LspNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for LspNoun {
    fn name(&self) -> &'static str {
        "lsp"
    }
    fn about(&self) -> &'static str {
        "Language server for local CI/CD readiness diagnostics"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![
            Box::new(LspServeVerb),
            Box::new(LspDoctorVerb),
            Box::new(LspExplainVerb),
        ]
    }
}

// ---------------------------------------------------------------------------
// Static diagnostic code catalog
// ---------------------------------------------------------------------------

struct CicdCodeEntry {
    code: &'static str,
    title: &'static str,
    severity: &'static str,
    observed: &'static str,
    repair: &'static str,
    clears_when: &'static str,
}

static CICD_CATALOG: &[CicdCodeEntry] = &[
    CicdCodeEntry {
        code: "CICD-GIT-001",
        title: "dirty_tree_blocks_close",
        severity: "Error",
        observed: "git index",
        repair: "run cargo cicd git status; cargo cicd git close",
        clears_when: "working tree is clean",
    },
    CicdCodeEntry {
        code: "CICD-GIT-002",
        title: "untracked_files_present",
        severity: "Warning",
        observed: "git index",
        repair: "stage or .gitignore untracked files, then run cargo cicd git close",
        clears_when: "no untracked files remain",
    },
    CicdCodeEntry {
        code: "CICD-GIT-003",
        title: "branch_behind_remote",
        severity: "Warning",
        observed: "git remote",
        repair: "run git pull --rebase to sync with remote",
        clears_when: "local branch is up to date with remote",
    },
    CicdCodeEntry {
        code: "CICD-PIPELINE-001",
        title: "pipeline_stage_failed",
        severity: "Error",
        observed: "pipeline run",
        repair: "run cargo cicd pipeline run and address reported stage failures",
        clears_when: "all pipeline stages pass",
    },
    CicdCodeEntry {
        code: "CICD-PIPELINE-002",
        title: "no_cicd_toml_found",
        severity: "Error",
        observed: "workspace root",
        repair: "run cargo cicd publish run to generate cicd.toml",
        clears_when: "cicd.toml exists at workspace root",
    },
    CicdCodeEntry {
        code: "CICD-TEST-001",
        title: "test_failures_block_close",
        severity: "Error",
        observed: "cargo test output",
        repair: "run cargo cicd test run and fix failing tests",
        clears_when: "all tests pass",
    },
    CicdCodeEntry {
        code: "CICD-WORKSPACE-001",
        title: "workspace_structure_invalid",
        severity: "Error",
        observed: "workspace Cargo.toml",
        repair: "run cargo cicd workspace doctor to diagnose structural issues",
        clears_when: "workspace doctor reports no violations",
    },
];

// ---------------------------------------------------------------------------
// Verbs
// ---------------------------------------------------------------------------

pub struct LspServeVerb;
impl VerbCommand for LspServeVerb {
    fn name(&self) -> &'static str {
        "serve"
    }
    fn about(&self) -> &'static str {
        "Start the cargo-cicd LSP server"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("lsp:serve");
        start_evt.case_id = Some(case_id.clone());

        println!("starting cargo-cicd-lsp server...");

        let verdict = match which_binary("cargo-cicd-lsp") {
            Some(path) => {
                println!("found: {}", path);
                println!("launching cargo-cicd-lsp (stdio transport)");
                let status = std::process::Command::new(&path).status();
                match status {
                    Ok(s) if s.success() => "PASS",
                    Ok(s) => {
                        eprintln!("cargo-cicd-lsp exited with: {}", s);
                        "FAIL"
                    }
                    Err(e) => {
                        eprintln!("failed to launch {}: {}", path, e);
                        "FAIL"
                    }
                }
            }
            None => {
                println!();
                println!("BLOCKED: cargo-cicd-lsp binary not found on PATH.");
                println!();
                println!("Install with:");
                println!("  cargo install cargo-cicd-lsp");
                println!();
                println!("Or build from source:");
                println!("  git clone https://github.com/your-org/cargo-cicd-lsp");
                println!("  cd cargo-cicd-lsp && cargo install --path .");
                "BLOCKED"
            }
        };

        let mut complete_evt = ProcessEvent::completed("lsp:serve", t0, verdict);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

pub struct LspDoctorVerb;
impl VerbCommand for LspDoctorVerb {
    fn name(&self) -> &'static str {
        "doctor"
    }
    fn about(&self) -> &'static str {
        "Check LSP health: binary presence, wpm oracle, workspace structure"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("lsp:doctor");
        start_evt.case_id = Some(case_id.clone());

        println!("lsp doctor");
        println!("==========");

        let mut all_ok = true;

        // Check 1: cargo-cicd-lsp binary
        match which_binary("cargo-cicd-lsp") {
            Some(path) => println!("[PASS] cargo-cicd-lsp binary: {}", path),
            None => {
                println!("[FAIL] cargo-cicd-lsp binary: not found on PATH");
                println!("       install: cargo install cargo-cicd-lsp");
                all_ok = false;
            }
        }

        // Check 2: wpm oracle
        match which_binary("wpm") {
            Some(path) => println!("[PASS] wpm oracle: {}", path),
            None => {
                println!("[WARN] wpm oracle: not found on PATH");
                println!("       some LSP diagnostics may be unavailable without wpm");
            }
        }

        // Check 3: workspace structure (Cargo.toml present)
        let cargo_toml = std::path::Path::new("Cargo.toml");
        if cargo_toml.exists() {
            println!("[PASS] workspace Cargo.toml: found");
        } else {
            println!("[FAIL] workspace Cargo.toml: not found in current directory");
            println!("       run from workspace root");
            all_ok = false;
        }

        // Check 4: cicd.toml
        let cicd_toml = std::path::Path::new("cicd.toml");
        if cicd_toml.exists() {
            println!("[PASS] cicd.toml: found");
        } else {
            println!("[WARN] cicd.toml: not found");
            println!("       run: cargo cicd publish run");
        }

        println!();
        let verdict = if all_ok { "PASS" } else { "FAIL" };
        println!("result: {}", verdict);

        let mut complete_evt = ProcessEvent::completed("lsp:doctor", t0, verdict);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

pub struct LspExplainVerb;
impl VerbCommand for LspExplainVerb {
    fn name(&self) -> &'static str {
        "explain"
    }
    fn about(&self) -> &'static str {
        "Explain a diagnostic code (e.g. CICD-GIT-001)"
    }
    fn trailing_var_arg(&self) -> bool {
        true
    }
    fn run(&self, args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("lsp:explain");
        start_evt.case_id = Some(case_id.clone());

        let codes = args.trailing();
        let code = codes.first().map(|s| s.to_ascii_uppercase()).unwrap_or_default();

        let verdict = match CICD_CATALOG.iter().find(|e| e.code == code) {
            Some(entry) => {
                println!("Code:         {}", entry.code);
                println!("Title:        {}", entry.title);
                println!("Severity:     {}", entry.severity);
                println!("Observed:     {}", entry.observed);
                println!("Repair:       {}", entry.repair);
                println!("Clears when:  {}", entry.clears_when);
                "PASS"
            }
            None => {
                eprintln!("unknown diagnostic code: {}", code);
                eprintln!();
                eprintln!("known codes:");
                for entry in CICD_CATALOG {
                    eprintln!("  {}  {}", entry.code, entry.title);
                }
                "FAIL"
            }
        };

        let mut complete_evt = ProcessEvent::completed("lsp:explain", t0, verdict);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return a human-readable explanation for a diagnostic code like "CICD-GIT-001".
pub fn explain_diagnostic_code(code: &str) -> String {
    match code {
        "CICD-GIT-001" => "dirty_tree_blocks_close: the working tree has uncommitted changes. Run 'cargo cicd git status' to inspect, then commit or stash before closing the phase.".to_string(),
        "CICD-GIT-002" => "untracked_artifacts: untracked files exist that may represent unintended output. Run 'cargo cicd git status' to review.".to_string(),
        "CICD-GIT-003" => "source_changed_after_close: source was modified after the last git close. Re-run the manufacturing pipeline.".to_string(),
        "CICD-EVIDENCE-001" => "evidence_missing: no process evidence directory found. Run a cargo cicd command to emit evidence.".to_string(),
        "CICD-EVIDENCE-002" => "stale_evidence: evidence is older than the last source change. Run 'cargo cicd test changed' and 'cargo cicd workspace doctor'.".to_string(),
        "CICD-EVIDENCE-003" => "hardcoded_timestamp: evidence contains a hardcoded timestamp instead of a real UTC time. Fix the emission code to use SystemTime::now().".to_string(),
        "CICD-EVIDENCE-004" => "missing_case_id: evidence events lack a session/case identifier. Ensure case_id is set on all emitted events.".to_string(),
        "CICD-EVIDENCE-005" => "receipt_before_court: a receipt was written before wpm adjudicated the evidence. Run 'cargo cicd evidence doctor' to adjudicate first.".to_string(),
        "CICD-WPM-001" => "unconfirmed_receipt_court: wpm binary not found or receipt doctor not confirmed. Install wasm4pm or set WPM_BIN env var.".to_string(),
        "CICD-WPM-002" => "capability_scan_missing: wpm capability scan has not been run. Run 'cargo cicd lsp doctor' to check.".to_string(),
        "CICD-WPM-003" => "runtime_court_not_invoked: wpm receipt doctor has not been called for the current evidence. Run 'cargo cicd evidence doctor'.".to_string(),
        "CICD-TEST-001" => "changed_test_not_run: changed test files have not been run. Run 'cargo cicd test changed'.".to_string(),
        "CICD-TEST-002" => "trybuild_fixture_changed: trybuild fixtures were modified. Re-run trybuild to confirm.".to_string(),
        "CICD-TARGET-001" => "target_growth_warning: target directory is large. Run 'cargo cicd target show' then 'cargo cicd target prune' if needed.".to_string(),
        "CICD-PUBLISH-001" => "dry_run_missing: cargo publish --dry-run has not been run. Run it before publishing.".to_string(),
        "CICD-PUBLISH-002" => "dry_run_without_receipt: publish dry-run completed but no receipt exists. Run 'cargo cicd evidence doctor' then 'cargo cicd publish'.".to_string(),
        "CICD-PUBLISH-003" => "package_changed_after_dry_run: package was modified after the last dry-run. Re-run cargo publish --dry-run.".to_string(),
        "CICD-PUBLIC-001" => "private_term_leak: a private/forbidden term was found in a public-facing document. Remove or replace the term.".to_string(),
        "CICD-PUBLIC-002" => "public_boundary_scan_stale: the public boundary scan is out of date. Re-run workspace doctor.".to_string(),
        "CICD-GGEN-001" => "rendered_surface_stale: a ggen-rendered surface is out of date. Run 'ggen sync' to regenerate.".to_string(),
        "CICD-GGEN-002" => "rendered_surface_drift: a ggen-rendered block differs from its source law. Run 'ggen sync' to realign.".to_string(),
        "CICD-GGEN-003" => "custom_region_missing: a ggen-managed file is missing its custom region markers. Add the expected custom block.".to_string(),
        "CICD-CLOSE-001" => "false_close_risk: one or more serious diagnostics are active. Resolve all Error-severity findings before claiming phase closure.".to_string(),
        "CICD-SPEC-001" => "spec_missing_for_change: changed files have no corresponding spec entry. Add a spec or plan entry for this change.".to_string(),
        "CICD-SPEC-002" => "task_done_without_evidence: a task is marked complete but no fresh evidence exists for it. Run the manufacturing pipeline to produce evidence.".to_string(),
        _ => format!("Unknown diagnostic code: {}. Run 'cargo cicd lsp doctor' to list active diagnostics.", code),
    }
}

fn which_binary(name: &str) -> Option<String> {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
