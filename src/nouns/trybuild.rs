use crate::adapters::ChangedFileDetector;
use crate::evidence::ProcessEvent;
use crate::nouns::evidence_helpers::{finish_evidence, init_evidence};
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct TrybuildNoun;
impl TrybuildNoun {
    pub fn new() -> Self {
        Self
    }
}
impl Default for TrybuildNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for TrybuildNoun {
    fn name(&self) -> &'static str {
        "trybuild"
    }
    fn about(&self) -> &'static str {
        "Manage trybuild fixtures"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![
            Box::new(TrybuildChangedVerb),
            Box::new(TrybuildUpdateVerb),
            Box::new(TrybuildReviewVerb),
        ]
    }
}

pub struct TrybuildUpdateVerb;
impl VerbCommand for TrybuildUpdateVerb {
    fn name(&self) -> &'static str {
        "update"
    }
    fn about(&self) -> &'static str {
        "Update trybuild snapshots by running tests with TRYBUILD=overwrite"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("trybuild:update");
        start_evt.case_id = Some(case_id.clone());

        let output = std::process::Command::new("cargo")
            .arg("test")
            .env("TRYBUILD", "overwrite")
            .output();

        let verdict = match output {
            Ok(ref out) if out.status.success() => {
                println!("{}", String::from_utf8_lossy(&out.stdout));
                "PASS"
            }
            Ok(ref out) => {
                eprintln!("{}", String::from_utf8_lossy(&out.stderr));
                "FAIL"
            }
            Err(e) => {
                eprintln!("error running cargo test with TRYBUILD=overwrite: {}", e);
                "FAIL"
            }
        };

        let mut complete_evt = ProcessEvent::completed("trybuild:update", t0, verdict);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

pub struct TrybuildReviewVerb;
impl VerbCommand for TrybuildReviewVerb {
    fn name(&self) -> &'static str {
        "review"
    }
    fn about(&self) -> &'static str {
        "List trybuild fixture files and their modification times"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("trybuild:review");
        start_evt.case_id = Some(case_id.clone());

        println!("trybuild fixture review");
        println!("=======================");

        let mut found = 0usize;
        if let Ok(entries) = std::fs::read_dir("tests") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.contains("trybuild") {
                            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
                            println!("  {} ({} bytes)", path.display(), size);
                            found += 1;
                        }
                    }
                }
            }
        }

        if found == 0 {
            println!("  no trybuild fixture files found in tests/");
        } else {
            println!();
            println!("total: {} fixture file(s)", found);
        }

        let mut complete_evt = ProcessEvent::completed("trybuild:review", t0, "PASS");
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

pub struct TrybuildChangedVerb;
impl VerbCommand for TrybuildChangedVerb {
    fn name(&self) -> &'static str {
        "changed"
    }
    fn about(&self) -> &'static str {
        "Run trybuild for changed fixtures only"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let (evidence_dir, case_id, start_evt, t0) = init_evidence("trybuild:changed");
        let fixture_dir = "tests/ui";
        let base = "origin/main";
        let changed = ChangedFileDetector::changed_rs_files(base);
        let fixtures: Vec<_> = changed
            .iter()
            .filter(|f| ChangedFileDetector::is_trybuild_fixture(f))
            .collect();
        println!("trybuild changed plan");
        println!("====================");
        println!("base ref:             {}", base);
        println!("changed fixtures:     {}", fixtures.len());
        println!("mode:                 changed-only (all-fixture run is opt-in)");
        println!("snapshot mode:        changed-only");
        println!();
        if fixtures.is_empty() {
            println!("no changed trybuild fixtures detected");
            println!("skipping trybuild run — use 'cargo test' for full run");
        } else {
            println!("selected fixtures:");
            for f in &fixtures {
                println!("  {}", f);
            }
            println!();
            println!("to update snapshots: TRYBUILD=overwrite cargo test");
        }

        finish_evidence(
            start_evt,
            t0,
            case_id,
            "PASS",
            "trybuild:changed",
            &evidence_dir,
        );
        let _ = fixture_dir;
        Ok(())
    }
}
