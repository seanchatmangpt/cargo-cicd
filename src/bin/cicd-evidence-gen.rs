//! cicd-evidence-gen — Generate the `[evidence]` TOML block for a published crate.
//!
//! # Usage
//!
//! ```text
//! cicd-evidence-gen <crate-name> <crate-version> [OPTIONS]
//! ```
//!
//! # Options
//!
//! ```text
//! --receipt <path>        Path to the wasm4pm receipt JSON file.
//! --evidence-dir <dir>    Path to the evidence directory produced by the CI/CD run.
//! --oracle-key <base64>   Base64-encoded oracle public key.
//! --standard <name>       Standard satisfied (repeatable, e.g. "IEC 61508 SIL 2").
//! --append <Cargo.toml>   Append the generated block to this file instead of printing it.
//! --check                 Print validation issues and exit 1 if any exist.
//! ```
//!
//! # Examples
//!
//! Print the [evidence] block for my-crate 1.0.0:
//!
//! ```text
//! cicd-evidence-gen my-crate 1.0.0
//! ```
//!
//! Generate with a receipt file, oracle key, and a standard:
//!
//! ```text
//! cicd-evidence-gen my-crate 1.0.0 \
//!     --receipt target/cargo-cicd/evidence/receipts/latest.json \
//!     --evidence-dir target/cargo-cicd/evidence \
//!     --oracle-key "base64encodedkey==" \
//!     --standard "IEC 61508 SIL 2"
//! ```
//!
//! Append to Cargo.toml (idempotent — overwrites existing [evidence] section if present):
//!
//! ```text
//! cicd-evidence-gen my-crate 1.0.0 --receipt latest.json --append Cargo.toml
//! ```

use cargo_cicd::evidence_manifest::{build_manifest, compute_trustworthiness};
use std::path::PathBuf;
use std::process;

// ─── Argument parsing (std-only, no clap needed for a thin binary) ────────────

#[derive(Debug)]
struct Args {
    crate_name: String,
    crate_version: String,
    receipt: Option<PathBuf>,
    evidence_dir: Option<PathBuf>,
    oracle_key: Option<String>,
    standards: Vec<String>,
    append: Option<PathBuf>,
    check: bool,
}

fn parse_args() -> Result<Args, String> {
    let raw: Vec<String> = std::env::args().collect();

    // Positional: prog <crate-name> <crate-version> [flags...]
    if raw.len() < 3 {
        return Err(format!(
            "Usage: {} <crate-name> <crate-version> [OPTIONS]\n\
             \n\
             Options:\n\
             \t--receipt <path>       Path to wasm4pm receipt JSON\n\
             \t--evidence-dir <dir>   Evidence output directory\n\
             \t--oracle-key <base64>  Oracle public key (base64)\n\
             \t--standard <name>      Standard satisfied (repeatable)\n\
             \t--append <Cargo.toml>  Append block to this file\n\
             \t--check                Validate and exit 1 if issues exist\n",
            raw[0]
        ));
    }

    let crate_name = raw[1].clone();
    let crate_version = raw[2].clone();

    let mut receipt: Option<PathBuf> = None;
    let mut evidence_dir: Option<PathBuf> = None;
    let mut oracle_key: Option<String> = None;
    let mut standards: Vec<String> = Vec::new();
    let mut append: Option<PathBuf> = None;
    let mut check = false;

    let mut i = 3usize;
    while i < raw.len() {
        match raw[i].as_str() {
            "--receipt" => {
                i += 1;
                if i >= raw.len() {
                    return Err("--receipt requires a path argument".to_string());
                }
                receipt = Some(PathBuf::from(&raw[i]));
            }
            "--evidence-dir" => {
                i += 1;
                if i >= raw.len() {
                    return Err("--evidence-dir requires a path argument".to_string());
                }
                evidence_dir = Some(PathBuf::from(&raw[i]));
            }
            "--oracle-key" => {
                i += 1;
                if i >= raw.len() {
                    return Err("--oracle-key requires a base64 argument".to_string());
                }
                oracle_key = Some(raw[i].clone());
            }
            "--standard" => {
                i += 1;
                if i >= raw.len() {
                    return Err("--standard requires a name argument".to_string());
                }
                standards.push(raw[i].clone());
            }
            "--append" => {
                i += 1;
                if i >= raw.len() {
                    return Err("--append requires a Cargo.toml path argument".to_string());
                }
                append = Some(PathBuf::from(&raw[i]));
            }
            "--check" => {
                check = true;
            }
            flag => {
                return Err(format!("Unknown flag: {}", flag));
            }
        }
        i += 1;
    }

    Ok(Args {
        crate_name,
        crate_version,
        receipt,
        evidence_dir,
        oracle_key,
        standards,
        append,
        check,
    })
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(2);
        }
    };

    // Build the manifest from available inputs.
    let mut manifest = build_manifest(
        &args.crate_name,
        &args.crate_version,
        args.receipt.as_deref(),
        args.evidence_dir.as_deref(),
        args.oracle_key.as_deref(),
    );

    // Add any user-supplied standards.
    manifest.standards_satisfied.extend(args.standards);

    // Recompute score after standards are added.
    let score = compute_trustworthiness(&manifest);
    manifest.trustworthiness_score = Some(score);

    // Validate the manifest if --check was requested.
    if args.check {
        let (valid, issues) = manifest.validate();
        if !issues.is_empty() {
            for issue in &issues {
                eprintln!("ISSUE: {}", issue);
            }
        }
        if !valid {
            eprintln!(
                "\nValidation FAILED for {}-{}. Fix the issues above and re-run.",
                args.crate_name, args.crate_version
            );
            process::exit(1);
        } else {
            eprintln!(
                "Validation PASSED for {}-{}. Trustworthiness score: {:.2}",
                args.crate_name, args.crate_version, score
            );
        }
    }

    let block = manifest.to_toml_block();

    match args.append {
        Some(ref cargo_toml_path) => {
            // Append (or update) the [evidence] block in the target Cargo.toml.
            match append_or_update_evidence_block(cargo_toml_path, &block) {
                Ok(()) => {
                    eprintln!("Written [evidence] block to {}", cargo_toml_path.display());
                }
                Err(e) => {
                    eprintln!(
                        "error: could not write to {}: {}",
                        cargo_toml_path.display(),
                        e
                    );
                    process::exit(1);
                }
            }
        }
        None => {
            // Print to stdout.
            print!("{}", block);
        }
    }
}

/// Append or update the `[evidence]` block in a `Cargo.toml` file.
///
/// If the file already has an `[evidence]` section it is replaced in-place.
/// Otherwise the block is appended at the end.
fn append_or_update_evidence_block(
    cargo_toml_path: &std::path::Path,
    new_block: &str,
) -> Result<(), String> {
    let existing = std::fs::read_to_string(cargo_toml_path).map_err(|e| format!("read: {}", e))?;

    let updated = if has_evidence_section(&existing) {
        replace_evidence_section(&existing, new_block)
    } else {
        // Append after a blank line.
        format!("{}\n{}", existing.trim_end(), new_block)
    };

    std::fs::write(cargo_toml_path, updated).map_err(|e| format!("write: {}", e))?;
    Ok(())
}

/// Returns `true` if the TOML content already has an `[evidence]` section.
fn has_evidence_section(content: &str) -> bool {
    content.lines().any(|l| l.trim() == "[evidence]")
}

/// Replace the existing `[evidence]` block with `new_block`.
///
/// Everything from `[evidence]` to the next `[...]` section (exclusive) is
/// replaced.  Content before and after is preserved.
fn replace_evidence_section(content: &str, new_block: &str) -> String {
    let mut before = String::new();
    let mut after = String::new();
    let mut in_evidence = false;
    let mut evidence_done = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if !in_evidence && !evidence_done {
            if trimmed == "[evidence]" {
                in_evidence = true;
                continue;
            }
            before.push_str(line);
            before.push('\n');
        } else if in_evidence {
            if trimmed.starts_with('[') && trimmed != "[evidence]" {
                in_evidence = false;
                evidence_done = true;
                after.push_str(line);
                after.push('\n');
            }
            // Drop old evidence lines.
        } else {
            after.push_str(line);
            after.push('\n');
        }
    }

    format!("{}\n{}{}", before.trim_end(), new_block, after)
}
