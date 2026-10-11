//! certification show — fail-closed contract (CCICD): with no verifiable
//! evidence commands available (empty TempDir repo), `cargo cicd certification
//! show` must NOT pass. It exits non-zero and renders UNVERIFIED / NON-PASSING
//! verdicts — honest absence is never admitted as PASS.
//!
//! Chicago: real compiled binary, real subprocess, assert on actual exit code
//! and output bytes.

use std::process::Command;
use tempfile::TempDir;

fn bin() -> std::ffi::OsString {
    let bin_name = std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME is set by cargo");
    std::env::var_os(format!("CARGO_BIN_EXE_{bin_name}")).unwrap_or_else(|| {
        panic!("CARGO_BIN_EXE_{bin_name} not set -- is there a [[bin]] named after the package?")
    })
}

#[test]
fn certification_show_fails_closed_on_empty_command_set() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = Command::new(bin())
        .args(["certification", "show"])
        .current_dir(repo.path())
        .output()
        .expect("binary should build and run");
    assert!(
        !out.status.success(),
        "certification show must FAIL CLOSED (non-zero exit) with no verifiable commands; got exit {:?}",
        out.status.code()
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("UNVERIFIED") || combined.contains("NON-PASSING"),
        "fail-closed output must name UNVERIFIED or NON-PASSING; got:\n{combined}"
    );
}
