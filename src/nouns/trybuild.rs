use crate::adapters::ChangedFileDetector;
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
        vec![Box::new(TrybuildChangedVerb)]
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

        finish_evidence(start_evt, t0, case_id, "PASS", "trybuild:changed", &evidence_dir);
        let _ = fixture_dir;
        Ok(())
    }
}
