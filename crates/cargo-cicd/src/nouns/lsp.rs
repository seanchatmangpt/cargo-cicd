use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct LspNoun;
impl LspNoun {
    pub fn new() -> Self {
        Self
    }

    pub fn run_doctor() -> anyhow::Result<()> {
        let matches = clap::Command::new("lsp").get_matches_from(vec!["lsp"]);
        let args = clap_noun_verb::VerbArgs::new(matches);
        DoctorVerb.run(&args).map_err(|e| anyhow::anyhow!("{}", e))
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
        "Language server and diagnostics helpers"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![
            Box::new(ServeVerb),
            Box::new(DoctorVerb),
            Box::new(ExplainVerb),
        ]
    }
}

// ── serve ────────────────────────────────────────────────────────────────────

pub struct ServeVerb;
impl VerbCommand for ServeVerb {
    fn name(&self) -> &'static str {
        "serve"
    }
    fn about(&self) -> &'static str {
        "Launch the cargo-cicd language server"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let result = std::process::Command::new("cargo-cicd-lsp").status();
        match result {
            Ok(status) => {
                std::process::exit(status.code().unwrap_or(1));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("cargo-cicd-lsp binary not found. Install: cargo install cargo-cicd-lsp");
            }
            Err(e) => {
                eprintln!("error launching cargo-cicd-lsp: {}", e);
                std::process::exit(1);
            }
        }
        Ok(())
    }
}

// ── doctor ───────────────────────────────────────────────────────────────────

pub struct DoctorVerb;
impl VerbCommand for DoctorVerb {
    fn name(&self) -> &'static str {
        "doctor"
    }
    fn about(&self) -> &'static str {
        "Check language server and tooling health"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        println!("lsp doctor");
        println!("==========");

        // Check for cargo-cicd-lsp binary
        let lsp_found = which_binary("cargo-cicd-lsp");
        println!("[{}] cargo-cicd-lsp", if lsp_found { "OK" } else { "WARN" });
        if !lsp_found {
            println!("      not found — install: cargo install cargo-cicd-lsp");
        }

        // Check wpm binary
        let wpm_path = "/Users/sac/wasm4pm/target/release/wpm";
        let wpm_found = std::path::Path::new(wpm_path).exists();
        println!(
            "[{}] wpm process auditor ({})",
            if wpm_found { "OK" } else { "WARN" },
            wpm_path
        );
        if !wpm_found {
            println!("      not found — build with: cargo build --release in wasm4pm");
        }

        // Check evidence directory
        let evidence_dir = crate::evidence::evidence_dir();
        let evidence_exists = evidence_dir.exists();
        println!(
            "[{}] evidence dir ({})",
            if evidence_exists { "OK" } else { "WARN" },
            evidence_dir.display()
        );
        if !evidence_exists {
            println!("      not found — run a command to create it");
        }

        println!();
        if lsp_found && wpm_found && evidence_exists {
            println!("lsp tooling is healthy");
        } else {
            println!("lsp tooling has warnings (see above)");
        }
        Ok(())
    }
}

fn which_binary(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ── explain ──────────────────────────────────────────────────────────────────

pub struct ExplainVerb;
impl VerbCommand for ExplainVerb {
    fn name(&self) -> &'static str {
        "explain"
    }
    fn about(&self) -> &'static str {
        "Explain a diagnostic code (e.g. lsp explain E0001)"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        // Read the diagnostic code from the raw command line (position after "explain")
        let raw: Vec<String> = std::env::args().collect();
        let code = raw
            .iter()
            .skip_while(|a| a.as_str() != "explain")
            .nth(1)
            .cloned();

        match code {
            Some(c) => {
                let explanation = cargo_cicd_core::diagnostics::explain_code(&c);
                println!("{}", explanation);
            }
            None => {
                println!("Usage: cargo cicd lsp explain <CODE>");
                println!();
                println!("Examples:");
                println!("  cargo cicd lsp explain E0001");
                println!("  cargo cicd lsp explain W0010");
            }
        }
        Ok(())
    }
}
