//! Chicago subprocess tests for the `gate repo` verb.
//!
//! Real binary, real temp git repos, real JSON/SARIF on stdout, real exit
//! codes. Gate measures components from observable repo state: a pristine
//! repo scores all-zero and fails (exit non-zero); seeding real evidence
//! artifacts flips the corresponding components to 1.

use assert_cmd::Command;
use tempfile::TempDir;

fn cicd() -> Command {
    Command::cargo_bin("cargo-cicd").unwrap()
}

/// Fresh git repo with a commit: gate must fail (exit non-zero), still emit
/// the v1 JSON report with a `q_release` key, and every component reads 0
/// (nothing is hardcoded to pass).
#[test]
fn gate_repo_json_pristine_repo_fails_with_all_zero_components() {
    let dir = TempDir::new().unwrap();
    run_git(dir.path(), &["init"]);
    run_git(dir.path(), &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "--allow-empty", "-m", "init"]);

    let out = cicd()
        .args(["gate", "repo", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(
        !out.status.success(),
        "pristine repo must fail the gate (q_release 0), got exit {:?}",
        out.status.code()
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let report: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("gate --json must emit JSON on stdout, got {e}: {stdout}"));
    assert!(
        report.get("q_release").is_some(),
        "report must carry q key (q_release), got: {report}"
    );
    assert_eq!(report["q_release"], 0, "pristine repo must score q_release 0");

    let components = &report["components"];
    for key in [
        "agent_boundary",
        "pre_tool_use",
        "trace_profile",
        "receipt",
        "ocel",
        "doctor",
        "gate",
        "playground_cheats_detected",
        "no_prose_standing",
        "v_cargo_cicd",
    ] {
        assert_eq!(
            components[key], 0,
            "component {key} must be 0 in a pristine repo, got: {report}"
        );
    }
}

/// `gate repo --sarif`: SARIF 2.1.0 document with the full rule catalog
/// (>= 19 rules) under runs[0].tool.driver.rules.
#[test]
fn gate_repo_sarif_emits_v2_1_0_with_full_rule_catalog() {
    let dir = TempDir::new().unwrap();
    run_git(dir.path(), &["init"]);

    let out = cicd()
        .args(["gate", "repo", "--sarif"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    let sarif: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("--sarif must emit JSON on stdout, got {e}: {stdout}"));
    assert_eq!(sarif["version"], "2.1.0", "SARIF version must be 2.1.0");
    let rules = sarif["runs"][0]["tool"]["driver"]["rules"]
        .as_array()
        .unwrap_or_else(|| panic!("missing runs[0].tool.driver.rules: {sarif}"));
    assert!(
        rules.len() >= 19,
        "expected >= 19 SARIF rules, got {}",
        rules.len()
    );
}

/// A repo seeded with a real `.agents/hooks.json` (pre-tool-use wired to
/// cargo-cicd.execute) and a non-empty `.cargo-cicd/ocel/events.jsonl` gets
/// agent_boundary=1, pre_tool_use=1, and ocel=1 — the measurement reads the
/// actual artifacts, not constants.
#[test]
fn gate_repo_seeded_evidence_flips_components_to_one() {
    let dir = TempDir::new().unwrap();
    run_git(dir.path(), &["init"]);

    std::fs::create_dir_all(dir.path().join(".agents")).unwrap();
    std::fs::write(
        dir.path().join(".agents/hooks.json"),
        r#"{"hooks": {"pre-tool-use": [{"command": "cargo-cicd.execute"}]}}"#,
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join(".cargo-cicd/ocel")).unwrap();
    std::fs::write(
        dir.path().join(".cargo-cicd/ocel/events.jsonl"),
        "{\"event_type\":\"PipelineStarted\",\"eventTypeId\":\"PipelineStarted\"}\n",
    )
    .unwrap();

    let out = cicd()
        .args(["gate", "repo", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    let report: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("gate --json must emit JSON, got {e}: {stdout}"));
    let components = &report["components"];
    assert_eq!(components["pre_tool_use"], 1, "seeded hooks.json must set pre_tool_use: {report}");
    assert_eq!(components["agent_boundary"], 1, "seeded hooks.json must set agent_boundary: {report}");
    assert_eq!(components["ocel"], 1, "non-empty OCEL journal must set ocel: {report}");
}

fn run_git(dir: &std::path::Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("git {:?} failed to spawn: {e}", args));
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}
