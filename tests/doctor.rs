//! Chicago subprocess tests for the `doctor` noun (src/nouns/doctor.rs).
//! Binary helper pattern from tests/version_parity_proof.rs: drive the real
//! compiled binary, assert on real exit codes, stdout JSON, and files on disk.

use std::path::Path;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_cargo-cicd")
}

fn run(args: &[&str], cwd: &Path) -> (Option<i32>, String, String) {
    let out = Command::new(bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("cargo-cicd binary should build and run");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The CLI wrapper may emit more than one JSON document on stdout (e.g. the
/// command's JSON output followed by a `null` for the `Result<()>` return).
/// Parse the FIRST JSON document.
fn first_json(stdout: &str) -> serde_json::Value {
    serde_json::Deserializer::from_str(stdout)
        .into_iter::<serde_json::Value>()
        .next()
        .expect("stdout must contain at least one JSON document")
        .expect("first JSON document on stdout must be valid")
}

#[test]
fn doctor_repo_json_is_parseable_and_exit_matches_counterexamples() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (code, stdout, _stderr) = run(&["doctor", "repo", "--repo", ".", "--json"], tmp.path());

    // Exit must be consistent with the reported counterexample count
    // (0 => success, non-empty => non-zero "Fraud detected").
    let parsed = first_json(&stdout);
    assert_eq!(parsed["schema"], "cargo-cicd.doctor.v1");
    let counterexamples = parsed["counterexamples"]
        .as_array()
        .expect("counterexamples key is an array");
    let q_doctor = parsed["q_doctor"].as_u64().expect("q_doctor key");
    assert_eq!(
        q_doctor,
        u64::from(counterexamples.is_empty()),
        "q_doctor must equal 1 iff counterexamples is empty"
    );
    if counterexamples.is_empty() {
        assert_eq!(code, Some(0), "empty counterexamples must exit 0");
    } else {
        assert_ne!(code, Some(0), "counterexamples present must exit non-zero");
    }
}

#[test]
fn doctor_write_baseline_writes_schema_correct_file() {
    let tmp = tempfile::TempDir::new().unwrap();
    let baseline_path = tmp.path().join("baseline.json");
    let (code, _stdout, _stderr) = run(
        &[
            "doctor",
            "repo",
            "--repo",
            ".",
            "--write-baseline",
            baseline_path.to_str().unwrap(),
        ],
        tmp.path(),
    );

    assert_eq!(code, Some(0), "--write-baseline must exit 0");
    let on_disk = std::fs::read_to_string(&baseline_path)
        .expect("baseline.json must exist on disk after --write-baseline");
    let parsed: serde_json::Value =
        serde_json::from_str(&on_disk).expect("baseline file must be valid JSON");
    assert_eq!(parsed["schema"], "cargo-cicd.doctor.baseline.v1");
    assert!(parsed["counterexamples"].is_array());
    assert!(parsed["timestamp"].is_string());
    assert!(parsed["git_commit"].is_string());
}

#[test]
fn doctor_diff_with_baseline_exits_zero_after_baseline() {
    let tmp = tempfile::TempDir::new().unwrap();
    let baseline_path = tmp.path().join("baseline.json");
    let (wcode, _, _) = run(
        &[
            "doctor",
            "repo",
            "--repo",
            ".",
            "--write-baseline",
            baseline_path.to_str().unwrap(),
        ],
        tmp.path(),
    );
    assert_eq!(wcode, Some(0));

    let (code, stdout, _stderr) = run(
        &["doctor", "diff", "--baseline", baseline_path.to_str().unwrap()],
        tmp.path(),
    );
    assert_eq!(code, Some(0), "diff against own baseline must exit 0");
    let parsed = first_json(&stdout);
    assert_eq!(parsed["schema"], "cargo-cicd.doctor.diff.v1");
    assert_eq!(parsed["q_diff"], 1, "no regressions against own baseline");
    assert!(
        parsed["new_counterexamples"].as_array().unwrap().is_empty(),
        "no new counterexamples against own baseline"
    );
}

#[test]
fn doctor_diff_without_baseline_fails_with_message() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (code, _stdout, stderr) = run(&["doctor", "diff"], tmp.path());
    assert_ne!(code, Some(0), "--diff without --baseline must exit non-zero");
    assert!(
        stderr.contains("--baseline"),
        "stderr must name the missing --baseline flag, got: {stderr:?}"
    );
}

#[test]
fn doctor_evidence_runs_or_documents_blockage() {
    // doctor evidence shells out to legacy evidence generation, which requires
    // the external wasm4pm binary. Assert the real contract: either it runs
    // (exit 0 with output) or it fails closed with a typed BLOCKED naming the
    // missing dependency — never a silent pass.
    let tmp = tempfile::TempDir::new().unwrap();
    let (code, stdout, stderr) = run(&["doctor", "evidence", "--json"], tmp.path());
    if code == Some(0) {
        assert!(
            !stdout.trim().is_empty(),
            "doctor evidence exit 0 must produce output"
        );
    } else {
        let combined = format!("{stdout}{stderr}");
        assert!(
            combined.contains("BLOCKED") && combined.contains("wpm"),
            "non-zero doctor evidence must fail closed with BLOCKED naming wpm, got: {combined:?}"
        );
    }
}

#[test]
fn adversarial_assert_true_file_triggers_fake_test_counterexample() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::write(tmp.path().join("fake.rs"), "fn foo() { assert!(true); }\n").unwrap();

    let (code, stdout, _stderr) = run(&["doctor", "repo", "--repo", ".", "--json"], tmp.path());
    assert_ne!(code, Some(0), "a file containing assert!(true) must exit non-zero");
    let parsed = first_json(&stdout);
    let counterexamples = parsed["counterexamples"].as_array().unwrap();
    assert!(
        counterexamples
            .iter()
            .any(|c| c.as_str().unwrap_or("").contains("fake_test")),
        "scanner must fire fake_test for assert!(true), got: {counterexamples:?}"
    );
}
