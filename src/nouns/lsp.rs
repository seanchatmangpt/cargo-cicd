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
    // EVIDENCE family
    CicdCodeEntry {
        code: "CICD-EVIDENCE-001",
        title: "evidence_missing",
        severity: "Error",
        observed: "target/cargo-cicd/evidence/",
        repair: "run any cargo cicd command to emit process evidence",
        clears_when: "evidence directory exists with at least one event",
    },
    CicdCodeEntry {
        code: "CICD-EVIDENCE-002",
        title: "stale_evidence",
        severity: "Warning",
        observed: "target/cargo-cicd/evidence/events.xes",
        repair: "run cargo cicd test changed; cargo cicd workspace doctor",
        clears_when: "evidence is fresher than last source change",
    },
    CicdCodeEntry {
        code: "CICD-EVIDENCE-003",
        title: "hardcoded_timestamp",
        severity: "Error",
        observed: "target/cargo-cicd/evidence/events.jsonl",
        repair: "fix evidence emission to use SystemTime::now() instead of a literal timestamp",
        clears_when: "all event timestamps are real UTC times",
    },
    CicdCodeEntry {
        code: "CICD-EVIDENCE-004",
        title: "missing_case_id",
        severity: "Warning",
        observed: "target/cargo-cicd/evidence/events.jsonl",
        repair: "ensure case_id is set on all emitted ProcessEvents (use session::read_or_create_session_id)",
        clears_when: "all events carry a case_id",
    },
    CicdCodeEntry {
        code: "CICD-EVIDENCE-005",
        title: "receipt_before_court",
        severity: "Error",
        observed: "target/cargo-cicd/evidence/receipts/latest.json",
        repair: "run cargo cicd evidence doctor to adjudicate evidence before writing a receipt",
        clears_when: "receipt timestamp is after the wpm adjudication timestamp",
    },
    // WPM family
    CicdCodeEntry {
        code: "CICD-WPM-001",
        title: "unconfirmed_receipt_court",
        severity: "Error",
        observed: "wpm binary / $WPM_BIN",
        repair: "install wasm4pm or set WPM_BIN env var; run cargo cicd evidence doctor",
        clears_when: "wpm receipt doctor --strict returns Admitted",
    },
    CicdCodeEntry {
        code: "CICD-WPM-002",
        title: "capability_scan_missing",
        severity: "Warning",
        observed: "wpm capability cache",
        repair: "run cargo cicd lsp doctor to probe wpm capabilities",
        clears_when: "capability cache is present and current",
    },
    CicdCodeEntry {
        code: "CICD-WPM-003",
        title: "runtime_court_not_invoked",
        severity: "Error",
        observed: "target/cargo-cicd/evidence/receipts/",
        repair: "run cargo cicd evidence doctor to invoke wpm receipt doctor",
        clears_when: "receipt exists and was adjudicated by wpm",
    },
    CicdCodeEntry {
        code: "CICD-WPM-004",
        title: "verdict_key_mismatch",
        severity: "Error",
        observed: "audit surface / verdict reader",
        repair: "align court output schema with audit reader — court emits overall_fitness; reader must read overall_fitness",
        clears_when: "regression fixture proves audit reads correct key without silent zero fallback",
    },
    // TARGET family
    CicdCodeEntry {
        code: "CICD-TARGET-001",
        title: "target_growth_warning",
        severity: "Warning",
        observed: "target/",
        repair: "run cargo cicd target show then cargo cicd target prune if needed",
        clears_when: "target directory size is below threshold",
    },
    CicdCodeEntry {
        code: "CICD-TARGET-002",
        title: "target_prune_requires_dry_run",
        severity: "Information",
        observed: "target/",
        repair: "run cargo cicd target prune to review; add --apply to execute",
        clears_when: "incremental artifacts removed or threshold no longer exceeded",
    },
    // PUBLISH family
    CicdCodeEntry {
        code: "CICD-PUBLISH-001",
        title: "dry_run_missing",
        severity: "Warning",
        observed: "publish state",
        repair: "run cargo publish --dry-run before publishing",
        clears_when: "dry-run passes cleanly",
    },
    CicdCodeEntry {
        code: "CICD-PUBLISH-002",
        title: "dry_run_without_receipt",
        severity: "Error",
        observed: "target/cargo-cicd/evidence/receipts/",
        repair: "run cargo cicd evidence doctor then cargo cicd publish",
        clears_when: "admitted receipt exists and dry-run passes",
    },
    CicdCodeEntry {
        code: "CICD-PUBLISH-003",
        title: "package_changed_after_dry_run",
        severity: "Warning",
        observed: "src/ or Cargo.toml",
        repair: "re-run cargo publish --dry-run after the change",
        clears_when: "dry-run timestamp is after last source change",
    },
    // PUBLIC BOUNDARY family
    CicdCodeEntry {
        code: "CICD-PUBLIC-001",
        title: "private_term_leak",
        severity: "Error",
        observed: "README.md or docs/",
        repair: "remove or replace the forbidden private term in the public-facing file",
        clears_when: "no forbidden terms found in public surfaces",
    },
    CicdCodeEntry {
        code: "CICD-PUBLIC-002",
        title: "public_boundary_scan_stale",
        severity: "Warning",
        observed: "public docs",
        repair: "run cargo cicd workspace doctor to refresh the public boundary scan",
        clears_when: "scan is current with no violations",
    },
    // GGEN family
    CicdCodeEntry {
        code: "CICD-GGEN-001",
        title: "rendered_surface_stale",
        severity: "Warning",
        observed: "docs/ or README.md",
        repair: "run ggen sync to regenerate rendered surfaces from source law",
        clears_when: "rendered output matches source law",
    },
    CicdCodeEntry {
        code: "CICD-GGEN-002",
        title: "rendered_surface_drift",
        severity: "Error",
        observed: "ggen-managed file",
        repair: "run ggen sync to realign rendered blocks with ontology source law",
        clears_when: "all ggen-managed blocks match their source law",
    },
    CicdCodeEntry {
        code: "CICD-GGEN-003",
        title: "custom_region_missing",
        severity: "Warning",
        observed: "ggen-managed file",
        repair: "add the expected <!-- BEGIN custom:name --> / <!-- END custom:name --> markers",
        clears_when: "all expected custom regions are present",
    },
    // CLOSE family
    CicdCodeEntry {
        code: "CICD-CLOSE-001",
        title: "false_close_risk",
        severity: "Error",
        observed: "aggregate workspace state",
        repair: "resolve all Error-severity diagnostics before claiming phase closure",
        clears_when: "no Error-severity diagnostics remain active",
    },
    // SPEC family
    CicdCodeEntry {
        code: "CICD-SPEC-001",
        title: "spec_missing_for_change",
        severity: "Warning",
        observed: "specs/ or .specify/",
        repair: "add a spec entry for this change or run /speckit.specify",
        clears_when: "spec entry exists for all changed files",
    },
    CicdCodeEntry {
        code: "CICD-SPEC-002",
        title: "task_done_without_evidence",
        severity: "Error",
        observed: "specs/*/tasks.md",
        repair: "run the manufacturing pipeline to produce fresh evidence for the task",
        clears_when: "admitted evidence exists for all completed tasks",
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
    fn additional_args(&self) -> Vec<clap::Arg> {
        vec![clap::Arg::new("code")
            .help("Diagnostic code to explain (e.g. CICD-GIT-001)")
            .required(false)]
    }
    fn run(&self, args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("lsp:explain");
        start_evt.case_id = Some(case_id.clone());

        let code = args
            .get_one_str_opt("code")
            .map(|s| s.to_ascii_uppercase())
            .unwrap_or_default();

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
        "CICD-WPM-004" => r#"Code:     CICD-WPM-004
Title:    verdict_key_mismatch
Severity: Error
Observed: audit surface / verdict reader
Repair:   align court output schema with audit reader — court emits overall_fitness; reader must read overall_fitness
Clears when: regression fixture proves audit reads correct key without silent zero fallback"#.to_string(),
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
