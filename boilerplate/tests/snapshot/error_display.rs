use insta::assert_snapshot;
use project_core::CoreError;

// ---------------------------------------------------------------------------
// Overview
// ---------------------------------------------------------------------------
// Each test snapshot-asserts the Display string for one CoreError variant.
//
// Why snapshot these?
// - Error messages are part of the public API — users read them in CI logs.
// - A careless refactor could change "workspace not found" to "workspace_not_found"
//   breaking scripts that parse the output.
// - Snapshotting catches these regressions immediately.
//
// Fixture values are chosen to be stable, short, and unambiguous so the
// snapshot files are readable without context.
// ---------------------------------------------------------------------------

/// Variant 1: WorkspaceNotFound
#[test]
fn workspace_not_found_display() {
    let err = CoreError::workspace_not_found(
        "no Cargo.toml in /home/user/project or any parent directory".to_string(),
    );
    assert_snapshot!(
        "workspace_not_found_display",
        format!("{}", err)
    );
}

/// Variant 2: ConfigInvalid
#[test]
fn config_invalid_display() {
    let err = CoreError::config_invalid(
        "git.base_ref".to_string(),
        "must be a non-empty branch name, got empty string".to_string(),
    );
    assert_snapshot!(
        "config_invalid_display",
        format!("{}", err)
    );
}

/// Variant 3: ProcessFailed
#[test]
fn process_failed_display() {
    let err = CoreError::process_failed(
        "cargo test --all".to_string(),
        101,
    );
    assert_snapshot!(
        "process_failed_display",
        format!("{}", err)
    );
}

/// Variant 4: InvariantViolated
#[test]
fn invariant_violated_display() {
    let err = CoreError::invariant_violated(
        "no_full_trybuild_by_default".to_string(),
        "trybuild ran the full fixture set without a changed-file filter".to_string(),
    );
    assert_snapshot!(
        "invariant_violated_display",
        format!("{}", err)
    );
}

/// Variant 5: IoError
#[test]
fn io_error_display() {
    let err = CoreError::io_error(
        "permission denied reading /var/run/cargo-cicd.lock".to_string(),
    );
    assert_snapshot!(
        "io_error_display",
        format!("{}", err)
    );
}

/// Variant 6: SerializationFailed
#[test]
fn serialization_failed_display() {
    let err = CoreError::serialization_failed(
        "TOML serialization error: value for key `state.git_phase` is not a string".to_string(),
    );
    assert_snapshot!(
        "serialization_failed_display",
        format!("{}", err)
    );
}

/// Variant 7: OracleUnavailable
#[test]
fn oracle_unavailable_display() {
    let err = CoreError::oracle_unavailable(
        "wpm binary not found on PATH; install wasm4pm to enable verdict adjudication".to_string(),
    );
    assert_snapshot!(
        "oracle_unavailable_display",
        format!("{}", err)
    );
}

/// Variant 8: EvidenceInvalid
#[test]
fn evidence_invalid_display() {
    let err = CoreError::evidence_invalid(
        "XES trace is missing required attribute case_id".to_string(),
    );
    assert_snapshot!(
        "evidence_invalid_display",
        format!("{}", err)
    );
}

// ---------------------------------------------------------------------------
// Edge-case: process_failed with negative exit code (-1 = killed by signal)
// ---------------------------------------------------------------------------

/// Variant ProcessFailed with code -1 (signal kill) — display must still
/// contain the code.
#[test]
fn process_failed_negative_code_display() {
    let err = CoreError::process_failed("cargo build".to_string(), -1);
    assert_snapshot!(
        "process_failed_negative_code_display",
        format!("{}", err)
    );
}

// ---------------------------------------------------------------------------
// Edge-case: empty payloads — must not panic
// ---------------------------------------------------------------------------

/// All variants with empty string payloads must produce a non-empty Display
/// and not panic.
#[test]
fn empty_payloads_display_without_panic() {
    let variants: Vec<(&str, String)> = vec![
        (
            "workspace_not_found_empty",
            format!("{}", CoreError::workspace_not_found(String::new())),
        ),
        (
            "config_invalid_empty",
            format!("{}", CoreError::config_invalid(String::new(), String::new())),
        ),
        (
            "process_failed_empty_cmd",
            format!("{}", CoreError::process_failed(String::new(), 0)),
        ),
        (
            "invariant_violated_empty",
            format!("{}", CoreError::invariant_violated(String::new(), String::new())),
        ),
        (
            "io_error_empty",
            format!("{}", CoreError::io_error(String::new())),
        ),
        (
            "serialization_failed_empty",
            format!("{}", CoreError::serialization_failed(String::new())),
        ),
        (
            "oracle_unavailable_empty",
            format!("{}", CoreError::oracle_unavailable(String::new())),
        ),
        (
            "evidence_invalid_empty",
            format!("{}", CoreError::evidence_invalid(String::new())),
        ),
    ];

    for (name, display_str) in &variants {
        assert!(
            !display_str.is_empty(),
            "empty-payload variant {} produced an empty Display string",
            name
        );
    }

    // Snapshot the combined output to catch formatting regressions.
    let combined = variants
        .iter()
        .map(|(name, s)| format!("{}: {}", name, s))
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!("empty_payloads_display", combined);
}
