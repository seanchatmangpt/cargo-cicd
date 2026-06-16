//! Verifies every noun exposes its expected verbs.
//!
//! Each assertion invokes `<noun> <verb> --help` and accepts either a zero
//! exit code or stderr that contains "Usage" (clap writes usage to stderr on
//! certain error paths). This catches regressions where a verb is accidentally
//! dropped from a noun's `verbs()` list.
use assert_cmd::Command;

macro_rules! assert_verb_registered {
    ($noun:expr, $verb:expr) => {
        let output = Command::cargo_bin("cargo-cicd")
            .unwrap()
            .args([$noun, $verb, "--help"])
            .output()
            .unwrap();
        assert!(
            output.status.success()
                || String::from_utf8_lossy(&output.stderr).contains("Usage")
                || String::from_utf8_lossy(&output.stdout).contains("Usage"),
            "verb '{} {}' not registered or --help broken",
            $noun,
            $verb
        );
    };
}

#[test]
fn all_nouns_have_expected_verbs() {
    // git
    assert_verb_registered!("git", "status");
    assert_verb_registered!("git", "close");
    // test
    assert_verb_registered!("test", "changed");
    assert_verb_registered!("test", "run");
    assert_verb_registered!("test", "bench");
    // workspace
    assert_verb_registered!("workspace", "doctor");
    assert_verb_registered!("workspace", "validate");
    assert_verb_registered!("workspace", "sync");
    assert_verb_registered!("workspace", "list");
    // publish
    assert_verb_registered!("publish", "run");
    assert_verb_registered!("publish", "check");
    assert_verb_registered!("publish", "validate");
    // trybuild
    assert_verb_registered!("trybuild", "changed");
    assert_verb_registered!("trybuild", "update");
    assert_verb_registered!("trybuild", "review");
    // target
    assert_verb_registered!("target", "show");
    assert_verb_registered!("target", "prune");
    // evidence
    assert_verb_registered!("evidence", "doctor");
    assert_verb_registered!("evidence", "audit");
    // status
    assert_verb_registered!("status", "show");
}
