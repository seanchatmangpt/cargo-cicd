//! src/nouns/sbom.rs — cargo cicd sbom verbs (inlined from the retired legacy layer; see CCICD-106)

use crate::evidence::ProcessEvent;

// ── generate verb ──────────────────────────────────────────────────────────────

pub struct SbomGenerateVerb;

impl SbomGenerateVerb {
    fn execute(&self) -> std::result::Result<(), clap_noun_verb::error::NounVerbError> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("sbom:generate");
        start_evt.case_id = Some(case_id.clone());

        println!("SBOM generation");
        println!("===============");

        let verdict;
        let typed_error: Option<String> = match std::process::Command::new("cargo")
            .args(["cyclonedx", "--format", "json", "--output-cdx", "sbom.json"])
            .output()
        {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!(
                    "[UNAVAILABLE] cargo-cyclonedx not installed — run `cargo install cargo-cyclonedx`"
                );
                verdict = "UNAVAILABLE";
                Some(
                    "TOOL_UNAVAILABLE: cargo-cyclonedx is not installed \
                     (run `cargo install cargo-cyclonedx`)"
                        .to_string(),
                )
            }
            Err(e) => {
                println!("[FAIL] error running cargo cyclonedx: {}", e);
                verdict = "FAIL";
                Some(format!("cargo cyclonedx failed to run: {}", e))
            }
            Ok(output) if !output.status.success() => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // cargo exists but the cyclonedx subcommand is missing — that is
                // "tool missing", not "tool ran and found issues".
                if stderr.contains("no such command") {
                    println!(
                        "[UNAVAILABLE] cargo-cyclonedx not installed — run `cargo install cargo-cyclonedx`"
                    );
                    verdict = "UNAVAILABLE";
                    Some(
                        "TOOL_UNAVAILABLE: cargo-cyclonedx is not installed \
                         (run `cargo install cargo-cyclonedx`)"
                            .to_string(),
                    )
                } else {
                    println!("[FAIL] cargo cyclonedx failed: {}", stderr.trim());
                    verdict = "FAIL";
                    Some(format!("cargo cyclonedx failed: {}", stderr.trim()))
                }
            }
            Ok(_) => {
                println!("[OK] SBOM generated at sbom.json");
                verdict = "PASS";
                None
            }
        };

        let mut complete_evt = ProcessEvent::completed("sbom:generate", t0, verdict);
        complete_evt.case_id = Some(case_id.clone());

        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        // Fail-closed: a missing tool is a typed Unavailable error (distinct
        // from "tool ran and found issues"), and either failure exits non-zero.
        match typed_error {
            Some(msg) => Err(clap_noun_verb::error::NounVerbError::execution_error(msg)),
            None => Ok(()),
        }
    }
}


// ── show verb ──────────────────────────────────────────────────────────────────

pub struct SbomShowVerb;

impl SbomShowVerb {
    fn execute(&self) -> std::result::Result<(), clap_noun_verb::error::NounVerbError> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("sbom:show");
        start_evt.case_id = Some(case_id.clone());

        println!("SBOM");
        println!("====");

        let sbom_path = std::path::Path::new("sbom.json");
        let verdict = if sbom_path.exists() {
            let size = std::fs::metadata(sbom_path).map(|m| m.len()).unwrap_or(0);
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
            println!("[WARN] No sbom.json found — run `cargo cicd sbom generate` first");
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


use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[verb("generate")]
/// Generate an SBOM using cargo-cyclonedx (CycloneDX JSON format).
pub fn cmd_generate() -> Result<()> {
    SbomGenerateVerb
        .execute()
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}

#[verb("show")]
/// Show the current SBOM (sbom.json) summary.
pub fn cmd_show() -> Result<()> {
    SbomShowVerb.execute()
}
