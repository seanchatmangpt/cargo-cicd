#[cfg(feature = "anti-llm-cheat")]
use crate::evidence::ProcessEvent;
use crate::legacy_nouns::evidence_helpers::{finish_evidence, init_evidence};
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
        #[allow(unused_mut)]
        let mut v: Vec<Box<dyn VerbCommand>> = vec![
            Box::new(LspServeVerb),
            Box::new(LspDoctorVerb),
            Box::new(LspExplainVerb),
        ];
        #[cfg(feature = "anti-llm-cheat")]
        v.push(Box::new(LspCheckVerb));
        v
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
        code: "CICD-TEST-002",
        title: "trybuild_fixture_changed",
        severity: "Warning",
        observed: "tests/",
        repair: "re-run trybuild to confirm fixtures still compile and fail as expected",
        clears_when: "trybuild run after fixture change with no unexpected passes or failures",
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
        observed: "target/cargo-cicd/evidence/events.ocel.json",
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
    CicdCodeEntry {
        code: "CICD-TESTS-001",
        title: "tests_stale_mapping",
        severity: "Warning",
        observed: "tests/changed_tests.rs",
        repair: "run 'cargo cicd test changed' to refresh the test-to-source mapping",
        clears_when: "test mapping is current with all changed source files",
    },
    CicdCodeEntry {
        code: "CICD-TESTS-002",
        title: "tests_impact_unknown",
        severity: "Warning",
        observed: "changed_files state in cicd.toml",
        repair: "run 'cargo cicd test run' to determine test impact of recent changes",
        clears_when: "all changed files have associated test coverage mapped",
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
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("lsp:serve");

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

        finish_evidence(start_evt, t0, case_id, verdict, "lsp:serve", &evidence_dir);
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
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("lsp:doctor");

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

        finish_evidence(start_evt, t0, case_id, verdict, "lsp:doctor", &evidence_dir);
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
    fn build_command(&self) -> clap::Command {
        clap::Command::new(self.name()).about(self.about()).arg(
            clap::Arg::new("code")
                .help("Diagnostic code to explain (e.g. CICD-GIT-001)")
                .required(false)
                .index(1),
        )
    }
    fn run(&self, args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("lsp:explain");

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
                finish_evidence(start_evt, t0, case_id, "FAIL", "lsp:explain", &evidence_dir);
                return Err(clap_noun_verb::error::NounVerbError::execution_error(
                    format!("unknown diagnostic code: {code}"),
                ));
            }
        };

        finish_evidence(
            start_evt,
            t0,
            case_id,
            verdict,
            "lsp:explain",
            &evidence_dir,
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// LspCheckVerb — anti-LLM admissibility scan (feature: anti-llm-cheat)
// ---------------------------------------------------------------------------

#[cfg(feature = "anti-llm-cheat")]
pub struct LspCheckVerb;

#[cfg(feature = "anti-llm-cheat")]
impl VerbCommand for LspCheckVerb {
    fn name(&self) -> &'static str {
        "check"
    }
    fn about(&self) -> &'static str {
        "Scan changed .rs files for anti-LLM admissibility violations"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("lsp:check");
        start_evt.case_id = Some(case_id.clone());

        let state = crate::engine::EngineState::from_workspace();
        let files = &state.changed_files.changed_rs_files;

        // `lsp-max-anti-cheat` is not published on crates.io and this crate must
        // never depend on an unpinned local/git source (see CLAUDE.md's
        // fix-forward / unpinned-foundation lesson). Until a versioned crate
        // exists, the scan is unavailable — treat that as a first-class Blocked
        // state (E7 philosophy), not a compile-time hard dependency.
        let all_diags: Vec<anti_llm_cheat_stub::AntiLlmDiagnostic> =
            anti_llm_cheat_stub::scan_files(files);

        if !anti_llm_cheat_stub::is_available() {
            println!("lsp check: diagnostic tool unavailable (lsp-max-anti-cheat not published; BLOCKED)");
        } else if all_diags.is_empty() {
            println!("lsp check: no admissibility violations found");
        } else {
            for d in &all_diags {
                let level = if d.blocking { "ERROR" } else { "WARN" };
                println!("[{}] {} — {}:{}", level, d.code, d.file_path, d.line);
                println!("      {}", d.message);
                if !d.required_correction.is_empty() {
                    println!("      fix: {}", d.required_correction);
                }
            }
        }

        let verdict = if !anti_llm_cheat_stub::is_available() {
            "WARN:oracle_unavailable"
        } else if all_diags.iter().any(|d| d.blocking) {
            "FAIL"
        } else if !all_diags.is_empty() {
            "WARN"
        } else {
            "PASS"
        };

        let mut complete_evt = ProcessEvent::completed("lsp:check", t0, verdict);
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

// ---------------------------------------------------------------------------
// anti_llm_cheat_stub — placeholder for the `lsp-max-anti-cheat` diagnostic
// tool, which is not published on crates.io. cargo-cicd must never depend on
// an unpinned local/git source for this, so `anti-llm-cheat` compiles against
// this always-unavailable stub instead. When a versioned crate is published,
// replace this module with a real `dep:lsp-max-anti-cheat` dependency and
// wire `anti-llm-cheat = ["dep:lsp-max-anti-cheat"]` in Cargo.toml.
// ---------------------------------------------------------------------------
#[cfg(feature = "anti-llm-cheat")]
mod anti_llm_cheat_stub {
    pub struct AntiLlmDiagnostic {
        pub code: String,
        pub file_path: String,
        pub line: u32,
        pub message: String,
        pub required_correction: String,
        pub blocking: bool,
    }

    /// Always false: the real scanner (`lsp-max-anti-cheat`) is not published
    /// on crates.io, so this feature is a first-class "Blocked" diagnostic
    /// tool, not a hard compile/runtime dependency.
    pub fn is_available() -> bool {
        false
    }

    pub fn scan_files(_files: &[String]) -> Vec<AntiLlmDiagnostic> {
        Vec::new()
    }
}
