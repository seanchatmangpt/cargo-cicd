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
    fn additional_args(&self) -> Vec<clap::Arg> {
        vec![clap::Arg::new("CODE")
            .help("Diagnostic code to explain (e.g. CICD-GIT-001)")
            .required(true)
            .index(1)]
    }
    fn run(&self, args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("lsp:explain");
        start_evt.case_id = Some(case_id.clone());

        let code = args
            .get_one_str_opt("CODE")
            .unwrap_or_default()
            .to_ascii_uppercase();

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
