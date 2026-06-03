#![allow(dead_code, unused_imports)]
use anyhow::Result;
use clap_noun_verb::CliBuilder;

mod adapters;
mod autonomic;
mod cicd_toml;
mod engine;
pub mod evidence;
mod integrations;
mod nouns;
mod policies;
pub mod session;
mod state;

// Inject default verbs so bare-noun invocations work:
//   cargo cicd status          → status show
//   cargo cicd publish         → publish run
//   cargo cicd workspace       → workspace doctor
// This preserves internal noun-verb grammar while exposing a simpler public surface.
fn inject_default_verbs(mut args: Vec<String>) -> Vec<String> {
    // Strip the Cargo external subcommand prefix: `cargo cicd status` → argv is
    // ["cargo-cicd", "cicd", "status"]; remove "cicd" so the binary sees ["cargo-cicd", "status"].
    if args.get(1).map(String::as_str) == Some("cicd") {
        args.remove(1);
    }
    // args[0] = binary name, args[1] = noun (maybe), args[2] = verb (maybe)
    let noun = args.get(1).map(String::as_str).unwrap_or("");
    let has_verb = args.get(2).map(|v| !v.starts_with('-')).unwrap_or(false);
    if !has_verb {
        let default_verb = match noun {
            "status" => Some("show"),
            "publish" => Some("run"),
            "workspace" => Some("doctor"),
            "evidence" => Some("doctor"),
            _ => None,
        };
        if let Some(verb) = default_verb {
            args.insert(2, verb.to_string());
        }
    }
    args
}

fn main() -> Result<()> {
    let raw: Vec<String> = std::env::args().collect();

    // Cargo external subcommand protocol: `cargo cicd status` invokes `cargo-cicd cicd status`.
    // Re-exec without the "cicd" prefix so the rest of main sees clean argv.
    if raw.get(1).map(String::as_str) == Some("cicd") {
        let bin = &raw[0];
        let rest = &raw[2..];
        let status = std::process::Command::new(bin).args(rest).status()?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Check bare-noun BEFORE verb injection: env::args() is what CliBuilder::run() reads.
    // If the user typed just the noun (no verb), dispatch directly to bypass the limitation.
    let noun = raw.get(1).map(String::as_str).unwrap_or("").to_string();
    let verb_arg = raw.get(2).map(String::as_str).unwrap_or("").to_string();
    let is_help_flag = matches!(verb_arg.as_str(), "--help" | "-h");
    let needs_default = matches!(
        noun.as_str(),
        "status" | "publish" | "workspace" | "evidence"
    ) && (verb_arg.is_empty() || (verb_arg.starts_with('-') && !is_help_flag));

    // Inject default verbs into local args (used only for reference, not for cli.run()).
    let _args = inject_default_verbs(raw);

    let cli = CliBuilder::new()
        .name("cargo-cicd")
        .version("26.6.2")
        .about("Local-first CI/CD helpers for Rust workspaces: clean target dirs, run changed tests, check git state, and publish cicd.toml.");

    if needs_default {
        match noun.as_str() {
            "status" => return nouns::status::StatusNoun::run_direct(),
            "publish" => return nouns::publish::PublishNoun::run_direct(),
            "workspace" => return nouns::workspace::WorkspaceNoun::run_doctor(),
            "evidence" => return nouns::evidence::EvidenceNoun::run_direct(),
            _ => {}
        }
    }

    let cli = cli
        .noun(nouns::evidence::EvidenceNoun::new())
        .noun(nouns::status::StatusNoun::new())
        .noun(nouns::target::TargetNoun::new())
        .noun(nouns::test::TestNoun::new())
        .noun(nouns::trybuild::TrybuildNoun::new())
        .noun(nouns::git::GitNoun::new())
        .noun(nouns::publish::PublishNoun::new())
        .noun(nouns::workspace::WorkspaceNoun::new());

    cli.run().map_err(|e| anyhow::anyhow!("{}", e))
}
