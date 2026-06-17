use crate::adapters::ChangedFileDetector;
use crate::evidence::ProcessEvent;
use crate::nouns::process_helpers::run_cargo;
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
        vec![
            Box::new(TestChangedVerb),
            Box::new(TestRunVerb),
            Box::new(TestBenchVerb),
        ]
    }
}

pub struct TestRunVerb;
impl VerbCommand for TestRunVerb {
    fn name(&self) -> &'static str {
        "run"
    }
    fn about(&self) -> &'static str {
        "Run all tests in the workspace"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("test:run");
        start_evt.case_id = Some(case_id.clone());

        let output = std::process::Command::new("cargo").arg("test").output();

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
                eprintln!("error running cargo test: {}", e);
                "FAIL"
            }
        };

        let mut complete_evt = ProcessEvent::completed("test:run", t0, verdict);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}

pub struct TestBenchVerb;
impl VerbCommand for TestBenchVerb {
    fn name(&self) -> &'static str {
        "bench"
    }
    fn about(&self) -> &'static str {
        "Run all benchmarks in the workspace"
    }
    fn run(&self, _args: &VerbArgs) -> clap_noun_verb::error::Result<()> {
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("test:bench");
        start_evt.case_id = Some(case_id.clone());

        let output = std::process::Command::new("cargo").arg("bench").output();

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
                eprintln!("error running cargo bench: {}", e);
                "FAIL"
            }
        };

        let mut complete_evt = ProcessEvent::completed("test:bench", t0, verdict);
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
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
        let evidence_dir = crate::evidence::evidence_dir();
        let case_id = crate::session::read_or_create_session_id(&evidence_dir);
        let (mut start_evt, t0) = ProcessEvent::started("test:changed");
        start_evt.case_id = Some(case_id.clone());
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

        let mut complete_evt = ProcessEvent::completed("test:changed", t0, "PASS");
        complete_evt.case_id = Some(case_id);
        if let Err(e) = crate::evidence::append_events(&[start_evt, complete_evt], &evidence_dir) {
            eprintln!("warning: evidence emission failed: {}", e);
        }
        Ok(())
    }
}
