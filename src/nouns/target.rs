use crate::adapters::TargetScannerAdapter;
use crate::evidence::ProcessEvent;
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
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("target:show");
        start_evt.case_id = Some(case_id.clone());

        let target_dir = "target";
        let size_gb = TargetScannerAdapter::total_size_gb(target_dir);
        let max_gb = 20.0_f64;
        let verdict_str = TargetScannerAdapter::verdict(size_gb, max_gb);
        println!("target directory: {}", target_dir);
        println!("total size:       {:.2} GB", size_gb);
        println!("max configured:   {:.1} GB", max_gb);
        println!("verdict:          {}", verdict_str);
        if verdict_str != "pass" {
            println!("recommendation:   run 'cargo cicd target prune' to free space");
        }

        // Count top-level artifacts in the target directory
        let _artifact_count = std::fs::read_dir(target_dir)
            .map(|rd| rd.count())
            .unwrap_or(0);
        let ev_verdict = if verdict_str == "pass" {
            "PASS"
        } else {
            "WARN"
        };
        let mut complete_evt = ProcessEvent::completed("target:show", t0, ev_verdict);
        complete_evt.case_id = Some(case_id.clone());

        let evidence_path = evidence_dir.join("events.xes");
        if let Err(e) = crate::evidence::emit_xes(&[start_evt, complete_evt], &evidence_path) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
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
    fn run(&self, args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);

        let (mut start_evt, t0) = ProcessEvent::started("target:prune");
        start_evt.case_id = Some(case_id.clone());

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

        let mut complete_evt = ProcessEvent::completed("target:prune", t0, verdict);
        complete_evt.case_id = Some(case_id.clone());

        let evidence_path = evidence_dir.join("events.xes");
        if let Err(e) = crate::evidence::emit_xes(&[start_evt, complete_evt], &evidence_path) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        let _ = (would_free_gb, release_protected);
        Ok(())
    }
}
