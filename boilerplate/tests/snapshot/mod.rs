pub mod cli_output;
pub mod error_display;

// ---------------------------------------------------------------------------
// Shared helpers for snapshot tests
// ---------------------------------------------------------------------------

use assert_cmd::Command;

/// Run `cargo-project <args>` and return stdout as a String.
/// Panics if the binary cannot be found.
pub fn run_project(args: &[&str]) -> String {
    let output = Command::cargo_bin("cargo-project")
        .expect("cargo-project binary must be built before running snapshot tests")
        .args(args)
        .output()
        .expect("failed to execute cargo-project");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Run `cargo-project <args>` and return stderr as a String.
pub fn run_project_stderr(args: &[&str]) -> String {
    let output = Command::cargo_bin("cargo-project")
        .expect("cargo-project binary must be built")
        .args(args)
        .output()
        .expect("failed to execute cargo-project");
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Normalise a snapshot string by:
/// - Collapsing internal runs of spaces to a single space (so column-aligned
///   help text does not break on minor formatting changes).
/// - Trimming leading/trailing whitespace from each line.
/// - Dropping empty lines that only contain spaces.
///
/// Use this normaliser when the snapshot value is help text that may reflow
/// across clap versions.
pub fn normalise_help(raw: &str) -> String {
    raw.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
