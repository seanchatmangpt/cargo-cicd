//! Chicago subprocess tests for the `release_gate` noun: real cicd.toml
//! config and real standing.json on disk in temp dirs, adjudicated by the
//! real binary's exit code and JSON report.

use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

/// The CLI harness prints the verb's JSON on one line, then a trailing
/// status line (e.g. `null`); parse the first JSON line only.
fn parse_first_json(stdout: &[u8]) -> serde_json::Value {
    let text = String::from_utf8_lossy(stdout);
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .expect("no JSON line on stdout");
    serde_json::from_str(line).expect("stdout JSON line should parse")
}

fn cicd(dir: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("cargo-cicd")
        .unwrap()
        .args(args)
        .current_dir(dir)
        .assert()
}

/// The CLI's default release id is `v{CARGO_PKG_VERSION}`; the gate config
/// key must match it.
fn default_release_id() -> String {
    format!("v{}", env!("CARGO_PKG_VERSION"))
}

fn write_cicd_toml(dir: &Path, release_id: &str, artifact_id: &str, status: &str) {
    // Minimal COMPLETE cicd.toml: `workspace`/`state`/`target`/etc. have no
    // serde defaults, so a gate-only file fails admission and the CLI
    // silently falls back to an empty config (required = 0).
    // BTreeMap<String, Vec<RequiredArtifactStatus>> → array-of-table syntax,
    // and the release-id key must be quoted (it contains dots).
    let toml = format!(
        concat!(
            "[workspace]\n",
            "name = \"gate-test\"\n",
            "toolchain = \"stable\"\n",
            "target_dir = \"target\"\n",
            "\n[state]\n",
            "dirty = false\n",
            "target_size_gb = 0.0\n",
            "changed_files = 0\n",
            "changed_tests = 0\n",
            "changed_trybuild_fixtures = 0\n",
            "\n[target]\n",
            "max_size_gb = 20\n",
            "prune_after_days = 14\n",
            "\n[test.changed]\n",
            "enabled = false\n",
            "base = \"main\"\n",
            "\n[trybuild.changed]\n",
            "enabled = false\n",
            "snapshot_mode = \"changed-only\"\n",
            "\n[git.phase]\n",
            "require_clean_tree = true\n",
            "commit_after_phase = false\n",
            "\n[autonomic]\n",
            "enabled = false\n",
            "mode = \"suggest\"\n",
            "\n[standing]\n",
            "ocel_logs = []\n",
            "process_validation = []\n",
            "receipt_ledgers = []\n",
            "\n[[standing.release_gates.\"{release_id}\"]]\n",
            "artifact_id = \"{artifact_id}\"\n",
            "status = \"{status}\"\n",
        ),
        release_id = release_id,
        artifact_id = artifact_id,
        status = status,
    );
    std::fs::write(dir.join("cicd.toml"), toml).unwrap();
}

fn write_standing_json(dir: &Path, artifact_id: &str, statuses: &[&str]) {
    let statuses_json: Vec<String> = statuses.iter().map(|s| format!("\"{}\"", s)).collect();
    let json = format!(
        r#"{{
  "schema_id": "cicd-standing.v1",
  "release_id": "test-release",
  "generated_at_utc": "2026-10-10T00:00:00Z",
  "generator": "release_gate_cli_test",
  "standing_version": "1",
  "artifacts": [
    {{
      "id": "{}",
      "kind": "rust_crate",
      "path": "{}",
      "standing": [{}],
      "ladder_level": 0,
      "evidence": [],
      "external_operator_side_effects": []
    }}
  ]
}}"#,
        artifact_id,
        artifact_id,
        statuses_json.join(", ")
    );
    let out = dir.join("target").join("praxis-standing");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("standing.json"), json).unwrap();
}

#[test]
fn no_config_no_standing_is_vacuously_satisfied_exit_zero() {
    let tmp = TempDir::new().unwrap();
    let out = cicd(tmp.path(), &["release_gate", "check", "--json"])
        .success()
        .get_output()
        .stdout
        .clone();
    let v = parse_first_json(&out);
    assert_eq!(v["schema"], "cargo-cicd.standing.release-gate.v1");
    assert_eq!(v["required"], 0);
    assert_eq!(v["q_release_gate"], 1);
}

#[test]
fn gate_requiring_absent_artifact_fails_with_nonzero_exit() {
    let tmp = TempDir::new().unwrap();
    write_cicd_toml(tmp.path(), &default_release_id(), "my-crate", "TESTED");
    // no standing.json seeded — requirement cannot be satisfied
    let out = cicd(tmp.path(), &["release_gate", "check", "--json"])
        .failure()
        .get_output()
        .stdout
        .clone();
    let v = parse_first_json(&out);
    assert_eq!(v["required"], 1);
    assert_eq!(v["satisfied"], 0);
    assert_eq!(v["q_release_gate"], 0);
    assert_eq!(v["missing"][0]["artifact_id"], "my-crate");
}

#[test]
fn gate_satisfied_by_seeded_standing_artifact_exits_zero() {
    let tmp = TempDir::new().unwrap();
    write_cicd_toml(tmp.path(), &default_release_id(), "my-crate", "TESTED");
    write_standing_json(tmp.path(), "my-crate", &["DISCOVERED", "BUILDS", "TESTED"]);
    let out = cicd(tmp.path(), &["release_gate", "check", "--json"])
        .success()
        .get_output()
        .stdout
        .clone();
    let v = parse_first_json(&out);
    assert_eq!(v["required"], 1);
    assert_eq!(v["satisfied"], 1);
    assert_eq!(v["q_release_gate"], 1);
    assert_eq!(v["missing"], serde_json::json!([]));
}
