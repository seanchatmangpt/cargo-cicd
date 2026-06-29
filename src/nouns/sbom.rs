//! Generate and show a Software Bill of Materials (SBOM) via CycloneDX.
// src/nouns/sbom.rs — cargo cicd sbom
//
// Thin wrapper delegating to the legacy implementation.
// See CCICD-106 for the deletion milestone.

#[allow(deprecated)]
pub use crate::legacy_nouns::sbom::{SbomGenerateVerb, SbomNoun, SbomShowVerb};

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[allow(deprecated)]
#[verb("generate")]
/// Generate an SBOM using cargo-cyclonedx (CycloneDX JSON format).
pub fn cmd_generate() -> Result<()> {
    SbomNoun::run_direct()
        .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(e.to_string()))
}

#[verb("show")]
/// Show the current SBOM (sbom.json).
pub fn cmd_show() -> Result<()> {
    show_sbom_impl()
}

fn show_sbom_impl() -> Result<()> {
    let sbom_path = std::path::Path::new("sbom.json");
    if sbom_path.exists() {
        let size = std::fs::metadata(sbom_path).map(|m| m.len()).unwrap_or(0);
        println!("[OK] CycloneDX SBOM present at sbom.json ({} bytes)", size);
        if let Ok(content) = std::fs::read_to_string(sbom_path) {
            let lines: Vec<&str> = content.lines().take(20).collect();
            println!("\n--- preview (first {} lines) ---", lines.len());
            for line in &lines {
                println!("{}", line);
            }
            if content.lines().count() > 20 {
                println!("... (truncated)");
            }
        }
    } else {
        println!("[WARN] No sbom.json found — run `cargo cicd sbom generate` first");
    }
    Ok(())
}
