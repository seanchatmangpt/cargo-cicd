//! misc noun coverage — CLI-meta subprocess tests.
//!
//! Every concrete noun (certification, claude_context, deploy, doctor,
//! evidence, gate, git, receipt, release_gate, standing, status, target,
//! trace, test, trybuild, workspace, verify, sbom, publish, ci, pipeline,
//! lsp, ocel, hooks) is owned by a dedicated tests/cli/*.rs lane target or
//! exclusion list. What remains for this target is the CLI surface itself:
//! top-level help must enumerate the full noun set, and unknown nouns must be
//! refused with a non-zero exit — never silently accepted.

use std::process::Command;
use tempfile::TempDir;

fn bin() -> std::ffi::OsString {
    let bin_name = std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME is set by cargo");
    std::env::var_os(format!("CARGO_BIN_EXE_{bin_name}")).unwrap_or_else(|| {
        panic!("CARGO_BIN_EXE_{bin_name} not set -- is there a [[bin]] named after the package?")
    })
}

fn run_cargo_cicd(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("binary should build and run")
}

const ALL_NOUNS: &[&str] = &[
    "certification",
    "ci",
    "claude_context",
    "deploy",
    "doctor",
    "evidence",
    "gate",
    "git",
    "hooks",
    "ocel", // lsp is feature-gated (anti-llm-cheat); covered by tests/cli/lsp.rs
    "pipeline",
    "publish",
    "receipt",
    "release_gate",
    "sbom",
    "standing",
    "status",
    "target",
    "test",
    "trace",
    "trybuild",
    "verify",
    "workspace",
];

#[test]
fn top_level_help_enumerates_all_nouns() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = Command::new(bin())
        .arg("--help")
        .current_dir(repo.path())
        .output()
        .expect("binary should build and run");
    assert!(
        out.status.success(),
        "top-level --help must exit 0; got {:?}",
        out.status.code()
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    for noun in ALL_NOUNS {
        assert!(
            stdout.contains(noun),
            "top-level help must list noun '{noun}'; got:\n{stdout}"
        );
    }
}

#[test]
fn unknown_noun_is_refused_nonzero() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = Command::new(bin())
        .args(["definitely-not-a-noun", "run"])
        .current_dir(repo.path())
        .output()
        .expect("binary should build and run");
    assert!(
        !out.status.success(),
        "an unknown noun must not be silently accepted; exit {:?}",
        out.status.code()
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.trim().is_empty(),
        "refusal must produce a diagnostic message"
    );
}

#[test]
fn each_default_feature_noun_accepts_its_primary_verb_help() {
    let repo = TempDir::new().expect("temp repo dir");
    for noun in ALL_NOUNS {
        let out = Command::new(bin())
            .args([noun, "--help"])
            .current_dir(repo.path())
            .output()
            .unwrap_or_else(|e| panic!("{noun} --help failed to spawn: {e}"));
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            out.status.success() || combined.contains("Usage"),
            "noun '{noun}' --help must succeed or print Usage; got {:?}:\n{combined}",
            out.status.code()
        );
    }
}

// ── ocel export / replay (basic contract) ────────────────────────────────────
// NOTE: if a dedicated tests/cli/ocel.rs lane target lands, move this module
// there. Kept here because no ocel.rs file is in this lane's owned set.

/// Seed a minimal process journal in the layout `crate::evidence::read_journal`
/// expects: one JSON line per `ProcessEvent` under `.cargo-cicd/evidence/events.jsonl`.
fn seed_journal(repo: &std::path::Path) {
    let dir = repo.join(".cargo-cicd").join("evidence");
    std::fs::create_dir_all(&dir).expect("seed evidence dir");
    std::fs::write(
        dir.join("events.jsonl"),
        concat!(
            r#"{"event_id":"evt-test-1","timestamp_iso":"2026-10-10T00:00:00Z","case_id":"case-1","lifecycle_transition":"complete","workspace_id":"cargo-cicd-workspace","repo_path":".","command":"ci:run","verdict_claimed":"PASS","duration_ms":10,"verdict_adjudicated":null,"adjudicated_at":null,"oracle_command":null,"trace_class":"live_workspace"}"#,
            "\n",
        ),
    )
    .expect("seed events.jsonl");
}

#[test]
fn ocel_export_from_seeded_journal_produces_structural_ocel_json() {
    let repo = TempDir::new().expect("temp repo dir");
    seed_journal(repo.path());
    let out_path = repo.path().join("out.ocel.json");
    let out = run_cargo_cicd(
        repo.path(),
        &[
            "ocel",
            "export",
            "--repo",
            repo.path().to_str().unwrap(),
            "--out",
            out_path.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "ocel export must exit 0 on a seeded journal; got {:?}",
        out.status.code()
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("OCEL export"),
        "export must report what it wrote; got:\n{stdout}"
    );
    let log: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out_path).expect("out.ocel.json exists"))
            .expect("export output must be valid JSON");
    for key in ["eventTypes", "events", "objects"] {
        assert!(
            log.get(key).map(|v| v.is_array()).unwrap_or(false),
            "OCEL 2.0 log must carry a `{key}` array; got: {log}"
        );
    }
    assert!(
        log["events"].as_array().map(|a| !a.is_empty()).unwrap_or(false),
        "the seeded event must be exported; got: {log}"
    );
}

#[test]
fn ocel_replay_with_no_journal_reports_typed_empty_status() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run_cargo_cicd(
        repo.path(),
        &[
            "ocel",
            "replay",
            "--json",
            "--repo",
            repo.path().to_str().unwrap(),
        ],
    );
    // Contract (src/ocel.rs replay_events): a missing journal is a typed
    // Empty summary — exit 0, status "empty", zero events verified, q=1.
    assert!(
        out.status.success(),
        "ocel replay with no journal must exit 0 with typed empty status; got {:?}",
        out.status.code()
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // The binary appends a stray trailing `null` line after the JSON; slice to
    // the outermost object before parsing.
    let start = stdout.find('{').expect("stdout must contain a JSON object");
    let end = stdout.rfind('}').expect("stdout must contain a JSON object") + 1;
    let parsed: serde_json::Value = serde_json::from_str(&stdout[start..end])
        .unwrap_or_else(|e| panic!("--json replay must print JSON ({e}); got:\n{stdout}"));
    assert_eq!(
        parsed["status"].as_str(),
        Some("empty"),
        "typed empty status expected; got: {parsed}"
    );
    assert_eq!(parsed["q"].as_i64(), Some(1), "empty is not a quality failure");
    assert_eq!(parsed["events_verified"].as_u64(), Some(0));
    assert_eq!(parsed["schema"].as_str(), Some("cargo-cicd.ocel.replay.v1"));
}

#[test]
fn ocel_replay_non_json_reports_no_events_found() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run_cargo_cicd(
        repo.path(),
        &["ocel", "replay", "--repo", repo.path().to_str().unwrap()],
    );
    assert!(out.status.success(), "must exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("no events log found"),
        "human output must say no events log was found; got:\n{stdout}"
    );
}
