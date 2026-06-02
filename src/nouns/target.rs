use crate::adapters::TargetScannerAdapter;
use crate::evidence::ProcessEvent;
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};
use std::time::Instant;

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
        let start = Instant::now();
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
        let duration_ms = start.elapsed().as_millis() as u64;
        let ev_verdict = if verdict_str == "pass" {
            "PASS"
        } else {
            "WARN"
        };
        let event = ProcessEvent::new("target show", ev_verdict);
        let evidence_path = crate::evidence::evidence_dir().join("events.xes");
        if let Err(e) = crate::evidence::emit_xes(&[event], &evidence_path) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        let _ = duration_ms;
        Ok(())
    }
}

pub struct TargetPruneVerb;
impl VerbCommand for TargetPruneVerb {
    fn name(&self) -> &'static str {
        "prune"
    }
    fn about(&self) -> &'static str {
        "Plan target directory cleanup (safe by default)"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let start = Instant::now();
        let target_dir = "target";
        let dry_run = true; // always suggest mode; --apply not yet wired
        let size_gb = TargetScannerAdapter::total_size_gb(target_dir);
        println!("target prune plan");
        println!("=================");
        println!("current size: {:.2} GB", size_gb);
        println!("mode:         suggest (use --apply to execute)");
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
        println!("to execute: cargo cicd target prune --apply");
        println!("note: release artifacts are never deleted automatically");

        let would_free_gb = (would_free_bytes as f64 / 1_073_741_824.0 * 100.0).round() / 100.0;
        let release_protected = std::path::Path::new(&format!("{}/release", target_dir)).exists();
        let duration_ms = start.elapsed().as_millis() as u64;
        let event = ProcessEvent::new("target prune", "PASS");
        let evidence_path = crate::evidence::evidence_dir().join("events.xes");
        if let Err(e) = crate::evidence::emit_xes(&[event], &evidence_path) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        let _ = (dry_run, duration_ms, would_free_gb, release_protected);
        Ok(())
    }
}
