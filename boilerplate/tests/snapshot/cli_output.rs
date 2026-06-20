use assert_cmd::Command;
use insta::assert_snapshot;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Run the binary with `args` and return stdout.  The test is skipped
/// (panics with a clear message) if the binary cannot be found so that
/// developers who haven't built the workspace yet get a clear failure instead
/// of an obscure one.
fn stdout(args: &[&str]) -> String {
    Command::cargo_bin("cargo-project")
        .expect(
            "cargo-project binary not found; run `cargo build` before snapshot tests",
        )
        .args(args)
        .output()
        .expect("failed to run cargo-project")
        .stdout
        .pipe(|b| String::from_utf8_lossy(b).into_owned())
}

/// Same as `stdout` but for commands that write their help/version to stderr
/// (clap can send --help to either stream depending on version).
fn stderr(args: &[&str]) -> String {
    Command::cargo_bin("cargo-project")
        .expect("cargo-project binary not found")
        .args(args)
        .output()
        .expect("failed to run cargo-project")
        .stderr
        .pipe(|b| String::from_utf8_lossy(b).into_owned())
}

/// Clap may write --help output to stdout OR stderr.  This helper returns
/// whichever is non-empty (stdout wins on a tie).
fn help_output(args: &[&str]) -> String {
    let out = Command::cargo_bin("cargo-project")
        .expect("cargo-project binary not found")
        .args(args)
        .output()
        .expect("failed to run cargo-project");

    let stdout_str = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr_str = String::from_utf8_lossy(&out.stderr).into_owned();

    if !stdout_str.trim().is_empty() {
        stdout_str
    } else {
        stderr_str
    }
}

// Trait to pipe Vec<u8> through a closure — avoids ugly intermediate bindings.
trait Pipe {
    fn pipe<F: FnOnce(&Self) -> String>(&self, f: F) -> String;
}
impl Pipe for Vec<u8> {
    fn pipe<F: FnOnce(&Self) -> String>(&self, f: F) -> String {
        f(self)
    }
}

// ---------------------------------------------------------------------------
// Snapshot: top-level --help
// ---------------------------------------------------------------------------

/// Snapshot the top-level `--help` output.
///
/// This catches:
/// - Accidental removal of a noun (status, workspace, …)
/// - Rewording of the mission statement
/// - Unexpected new subcommands appearing without review
///
/// To update the snapshot after an intentional change, run:
///   `cargo test --test snapshot -- snapshot::cli_output::top_level_help --update`
/// or set `INSTA_UPDATE=always` environment variable.
#[test]
fn top_level_help() {
    let output = help_output(&["--help"]);
    assert_snapshot!("top_level_help", output);
}

// ---------------------------------------------------------------------------
// Snapshot: status show --help
// ---------------------------------------------------------------------------

/// Snapshot `status show --help`.
///
/// Catches changes to the status noun's description, flags, and argument list.
#[test]
fn status_show_help() {
    let output = help_output(&["status", "show", "--help"]);
    assert_snapshot!("status_show_help", output);
}

// ---------------------------------------------------------------------------
// Snapshot: workspace doctor --help
// ---------------------------------------------------------------------------

/// Snapshot `workspace doctor --help`.
///
/// Catches changes to the workspace doctor verb description and flags.
#[test]
fn workspace_doctor_help() {
    let output = help_output(&["workspace", "doctor", "--help"]);
    assert_snapshot!("workspace_doctor_help", output);
}

// ---------------------------------------------------------------------------
// Snapshot: status --help  (noun-level, not verb-level)
// ---------------------------------------------------------------------------

/// Snapshot `status --help` (the noun level, which lists available verbs).
#[test]
fn status_noun_help() {
    let output = help_output(&["status", "--help"]);
    assert_snapshot!("status_noun_help", output);
}

// ---------------------------------------------------------------------------
// Snapshot: workspace --help  (noun-level)
// ---------------------------------------------------------------------------

/// Snapshot `workspace --help`.
#[test]
fn workspace_noun_help() {
    let output = help_output(&["workspace", "--help"]);
    assert_snapshot!("workspace_noun_help", output);
}

// ---------------------------------------------------------------------------
// Snapshot: --version
// ---------------------------------------------------------------------------

/// For `--version` we do NOT snapshot the full string (it changes with every
/// release).  Instead we assert that the output contains a semver-looking
/// token (`<major>.<minor>.<patch>`), then snapshot a *normalised* form where
/// the actual version number is replaced with `<VERSION>`.
///
/// This ensures the version flag works and the binary name is correct, without
/// the snapshot itself becoming stale on every bump.
#[test]
fn version_flag_contains_version_number() {
    let output = help_output(&["--version"]);

    // Must contain something that looks like a version number.
    let has_version = output
        .split_whitespace()
        .any(|token| token.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
            && token.contains('.'));

    assert!(
        has_version,
        "`--version` output does not contain a version number: {:?}",
        output
    );

    // Snapshot the normalised form.
    let normalised = output
        .split_whitespace()
        .map(|token| {
            // Replace tokens that look like semver (digits.digits.anything).
            if token.starts_with(|c: char| c.is_ascii_digit()) && token.contains('.') {
                "<VERSION>".to_string()
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    assert_snapshot!("version_normalised", normalised);
}

// ---------------------------------------------------------------------------
// Snapshot: default invocation (no subcommand — should print help or error)
// ---------------------------------------------------------------------------

/// Snapshot the output of `cargo-project` with no arguments.
///
/// The binary should either print top-level help or a "missing subcommand"
/// error — not crash.  The snapshot keeps this behaviour stable.
#[test]
fn no_args_output() {
    let out = Command::cargo_bin("cargo-project")
        .expect("binary not found")
        .output()
        .expect("failed to run");

    // Combine stdout + stderr since clap may use either.
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    assert_snapshot!("no_args_output", combined);
}
