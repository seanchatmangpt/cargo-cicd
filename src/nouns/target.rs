use crate::adapters::TargetScannerAdapter;
use crate::nouns::evidence_helpers::{finish_evidence, init_evidence};
use crate::ui::badge::{self, Verdict};
use crate::ui::theme::{self, Role};
use crate::ui::{chart, panel};
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct TargetNoun;
impl TargetNoun {
    pub fn new() -> Self {
        Self
    }
}
impl Default for TargetNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for TargetNoun {
    fn name(&self) -> &'static str {
        "target"
    }
    fn about(&self) -> &'static str {
        "Manage target directory"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(TargetShowVerb), Box::new(TargetPruneVerb)]
    }
}

pub struct TargetShowVerb;
impl VerbCommand for TargetShowVerb {
    fn name(&self) -> &'static str {
        "show"
    }
    fn about(&self) -> &'static str {
        "Show target directory size and state"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("target:show");

        let target_dir = "target";
        let size_gb = TargetScannerAdapter::total_size_gb(target_dir);
        let max_gb = 20.0_f64;
        let verdict_str = TargetScannerAdapter::verdict(size_gb, max_gb);

        println!("{}", panel::header("target directory state"));

        // Aligned key/value with a live gauge. "target directory" and the "GB"
        // readouts remain contiguous plain-text substrings off-TTY.
        let dir_v = theme::paint(target_dir, Role::Value);
        let usage_v = format!(
            "{}  {:.2} / {:.1} GB",
            chart::gauge(size_gb, max_gb, 18),
            size_gb,
            max_gb
        );
        let verdict_v = badge::tag(Verdict::from_tag(verdict_str));
        let mut rows: Vec<(&str, String)> = vec![
            ("target directory", dir_v),
            ("usage", usage_v),
            ("verdict", verdict_v),
        ];
        if verdict_str != "pass" {
            rows.push((
                "recommendation",
                theme::paint("run 'cargo cicd target prune' to free space", Role::Warning),
            ));
        }
        let rows_ref: Vec<(&str, &str)> = rows.iter().map(|(k, v)| (*k, v.as_str())).collect();
        println!("{}", panel::kv(&rows_ref));

        // Count top-level artifacts in the target directory
        let _artifact_count = std::fs::read_dir(target_dir)
            .map(|rd| rd.count())
            .unwrap_or(0);
        let ev_verdict = if verdict_str == "pass" { "PASS" } else { "WARN" };
        finish_evidence(start_evt, t0, case_id, ev_verdict, "target:show", &evidence_dir);
        Ok(())
    }
}

pub struct TargetPruneVerb;
impl VerbCommand for TargetPruneVerb {
    fn name(&self) -> &'static str {
        "prune"
    }
    fn about(&self) -> &'static str {
        "Plan target directory cleanup (safe by default; use --apply to execute)"
    }
    fn build_command(&self) -> clap::Command {
        clap::Command::new(self.name())
            .about(self.about())
            .arg(
                clap::Arg::new("apply")
                    .long("apply")
                    .action(clap::ArgAction::SetTrue)
                    .help("Execute the prune (delete incremental artifacts)"),
            )
            .arg(
                clap::Arg::new("dry-run")
                    .long("dry-run")
                    .action(clap::ArgAction::SetTrue)
                    .help("Show what would be deleted without deleting (default behavior)"),
            )
    }
    fn run(&self, args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("target:prune");

        // Respect --apply flag: without it this is a dry run (suggest mode only).
        // We detect --apply directly from the process args rather than via the
        // clap ArgMatches, because VerbArgs does not expose the flag declaration
        // layer needed to pre-register arbitrary boolean flags.
        let _ = args; // args used for other verbs; prune reads raw env args
        let apply = std::env::args().any(|a| a == "--apply");
        let dry_run = !apply;

        let target_dir = "target";
        let size_gb = TargetScannerAdapter::total_size_gb(target_dir);
        println!("target prune plan");
        println!("=================");
        println!("current size: {:.2} GB", size_gb);
        println!(
            "mode:         {}",
            if dry_run {
                "suggest (use --apply to execute)"
            } else {
                "apply (deleting incremental artifacts)"
            }
        );
        println!();
        println!("suggested candidates:");
        let mut would_free_bytes: u64 = 0;
        for profile in &["debug/incremental", "debug/.fingerprint", "debug/deps"] {
            let p = format!("{}/{}", target_dir, profile);
            if std::path::Path::new(&p).exists() {
                let sz = TargetScannerAdapter::total_size_bytes(&p);
                would_free_bytes += sz;
                println!("  {} ({:.2} GB)", p, sz as f64 / 1_073_741_824.0);
            }
        }
        println!();

        let would_free_gb = (would_free_bytes as f64 / 1_073_741_824.0 * 100.0).round() / 100.0;
        let release_protected = std::path::Path::new(&format!("{}/release", target_dir)).exists();

        let verdict = if dry_run {
            println!("to execute: cargo cicd target prune --apply");
            println!("note: release artifacts are never deleted automatically");
            // Dry-run is a planning step, not a completion — emit WARN:dry_run so the
            // evidence log accurately reflects that no action was taken.
            "WARN:dry_run"
        } else {
            // Actually remove the incremental build artifacts.
            let mut freed_bytes: u64 = 0;
            for profile in &["debug/incremental", "debug/.fingerprint", "debug/deps"] {
                let p = format!("{}/{}", target_dir, profile);
                if std::path::Path::new(&p).exists() {
                    let sz = TargetScannerAdapter::total_size_bytes(&p);
                    if std::fs::remove_dir_all(&p).is_ok() {
                        freed_bytes += sz;
                        println!("  removed {}", p);
                    } else {
                        eprintln!("  warning: could not remove {}", p);
                    }
                }
            }
            let freed_gb = (freed_bytes as f64 / 1_073_741_824.0 * 100.0).round() / 100.0;
            println!();
            println!("freed: {:.2} GB", freed_gb);
            println!("note: release artifacts are never deleted automatically");
            "PASS"
        };

        finish_evidence(start_evt, t0, case_id, verdict, "target:prune", &evidence_dir);
        let _ = (would_free_gb, release_protected);
        Ok(())
    }
}
