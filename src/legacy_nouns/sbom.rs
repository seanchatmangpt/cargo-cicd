use crate::evidence::ProcessEvent;
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct SbomNoun;

impl SbomNoun {
    pub fn new() -> Self {
        Self
    }

    pub fn run_direct() -> anyhow::Result<()> {
        SbomGenerateVerb
            .execute()
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}

impl Default for SbomNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for SbomNoun {
    fn name(&self) -> &'static str {
        "sbom"
    }
    fn about(&self) -> &'static str {
        "Generate and show a Software Bill of Materials (SBOM) via CycloneDX"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(SbomGenerateVerb), Box::new(SbomShowVerb)]
    }
}

// ── generate verb ──────────────────────────────────────────────────────────────

pub struct SbomGenerateVerb;

impl SbomGenerateVerb {
    fn execute(&self) -> Result<(), clap_noun_verb::error::NounVerbError> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("sbom:generate");
        start_evt.case_id = Some(case_id.clone());

        println!("SBOM generation");
        println!("===============");

        let verdict = match std::process::Command::new("cargo")
            .args(["cyclonedx", "--format", "json", "--output-cdx", "sbom.json"])
            .output()
        {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!(
                    "[WARN] cargo-cyclonedx not installed — run `cargo install cargo-cyclonedx`"
                );
                "WARN:cyclonedx_unavailable"
            }
            Err(e) => {
                println!("[FAIL] error running cargo cyclonedx: {}", e);
                "FAIL"
            }
            Ok(output) if !output.status.success() => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("[FAIL] cargo cyclonedx failed: {}", stderr.trim());
                "FAIL"
            }
            Ok(_) => {
                println!("[OK] SBOM generated at sbom.json");
                "PASS"
            }
        };

        let mut complete_evt = ProcessEvent::completed("sbom:generate", t0, verdict);
        complete_evt.case_id = Some(case_id.clone());

        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

impl VerbCommand for SbomGenerateVerb {
    fn name(&self) -> &'static str {
        "generate"
    }
    fn about(&self) -> &'static str {
        "Generate an SBOM using cargo-cyclonedx (CycloneDX JSON format)"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        self.execute()
    }
}

// ── show verb ──────────────────────────────────────────────────────────────────

pub struct SbomShowVerb;

impl SbomShowVerb {
    fn execute(&self) -> Result<(), clap_noun_verb::error::NounVerbError> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("sbom:show");
        start_evt.case_id = Some(case_id.clone());

        println!("SBOM");
        println!("====");

        let sbom_path = std::path::Path::new("sbom.json");
        let verdict = if sbom_path.exists() {
            let size = std::fs::metadata(sbom_path)
                .map(|m| m.len())
                .unwrap_or(0);
            println!("[OK] CycloneDX SBOM present at sbom.json ({} bytes)", size);

            // Print the first 20 lines as a preview.
            if let Ok(content) = std::fs::read_to_string(sbom_path) {
                let lines: Vec<&str> = content.lines().take(20).collect();
                println!();
                println!("--- preview (first {} lines) ---", lines.len());
                for line in &lines {
                    println!("{}", line);
                }
                if content.lines().count() > 20 {
                    println!("... (truncated)");
                }
            }
            "PASS"
        } else {
            println!(
                "[WARN] No sbom.json found — run `cargo cicd sbom generate` first"
            );
            "WARN"
        };

        let mut complete_evt = ProcessEvent::completed("sbom:show", t0, verdict);
        complete_evt.case_id = Some(case_id.clone());

        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

impl VerbCommand for SbomShowVerb {
    fn name(&self) -> &'static str {
        "show"
    }
    fn about(&self) -> &'static str {
        "Show the current SBOM (sbom.json) summary"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        self.execute()
    }
}
