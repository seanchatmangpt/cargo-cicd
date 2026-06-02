use crate::adapters::ChangedFileDetector;
use clap_noun_verb::{NounCommand, VerbArgs, VerbCommand};

pub struct TestNoun;
impl TestNoun {
    pub fn new() -> Self {
        Self
    }
}
impl Default for TestNoun {
    fn default() -> Self {
        Self::new()
    }
}

impl NounCommand for TestNoun {
    fn name(&self) -> &'static str {
        "test"
    }
    fn about(&self) -> &'static str {
        "Run changed tests"
    }
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
        vec![Box::new(TestChangedVerb)]
    }
}

pub struct TestChangedVerb;
impl VerbCommand for TestChangedVerb {
    fn name(&self) -> &'static str {
        "changed"
    }
    fn about(&self) -> &'static str {
        "Run tests for changed files only"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let base = "origin/main";
        let changed = ChangedFileDetector::changed_rs_files(base);
        let test_files: Vec<_> = changed
            .iter()
            .filter(|f| ChangedFileDetector::is_test_file(f))
            .collect();
        println!("changed test plan");
        println!("=================");
        println!("base ref:         {}", base);
        println!("changed .rs:      {}", changed.len());
        println!("affected tests:   {}", test_files.len());
        if test_files.is_empty() {
            println!("no changed test files detected — conservative mode");
            println!("recommendation: run 'cargo test' to be safe");
        } else {
            for t in &test_files {
                println!("  {}", t);
            }
            println!();
            let joined: Vec<&str> = test_files.iter().map(|s| s.as_str()).collect();
            println!("run: cargo test {}", joined.join(" "));
        }
        println!();
        println!("note: exact affected-test selection is conservative by design");
        Ok(())
    }
}
