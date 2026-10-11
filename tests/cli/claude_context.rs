//! claude_context show — Chicago subprocess tests against the real binary.
//!
//! (a) No standing.json -> typed non-zero failure naming the missing file and
//!     the `standing refresh` remediation.
//! (b) Seeded minimal StandingDocument -> exit 0, prints the
//!     `# CLAUDE_CODE_CONTEXT` header, writes CLAUDE_CODE_CONTEXT.md.

use std::process::Command;
use tempfile::TempDir;

fn bin() -> std::ffi::OsString {
    let bin_name = std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME is set by cargo");
    std::env::var_os(format!("CARGO_BIN_EXE_{bin_name}")).unwrap_or_else(|| {
        panic!("CARGO_BIN_EXE_{bin_name} not set -- is there a [[bin]] named after the package?")
    })
}

fn run(repo: &std::path::Path) -> std::process::Output {
    Command::new(bin())
        .args(["claude_context", "show"])
        .current_dir(repo)
        .output()
        .expect("binary should build and run")
}

#[test]
fn missing_standing_json_fails_with_refresh_hint() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = run(repo.path());
    assert!(
        !out.status.success(),
        "claude_context show without standing.json must exit non-zero; got {:?}",
        out.status.code()
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("no standing.json") || combined.contains("standing refresh"),
        "error must name the missing standing.json and the `standing refresh` remediation; got:\n{combined}"
    );
}

#[test]
fn seeded_standing_document_renders_context_and_writes_file() {
    let repo = TempDir::new().expect("temp repo dir");
    let standing_dir = repo.path().join("target").join("praxis-standing");
    std::fs::create_dir_all(&standing_dir).expect("create standing dir");
    // Minimal StandingDocument: schema_id defaults; empty artifacts render the
    // header only — that is the whole observable contract for exit-0.
    std::fs::write(
        standing_dir.join("standing.json"),
        r#"{
            "release_id": "test-release",
            "generated_at_utc": "2026-10-10T00:00:00Z",
            "generator": "claude_context-test",
            "standing_version": "1",
            "artifacts": []
        }"#,
    )
    .expect("seed standing.json");

    let out = run(repo.path());
    assert!(
        out.status.success(),
        "claude_context show with valid standing.json must exit 0; got {:?}",
        out.status.code()
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("# CLAUDE_CODE_CONTEXT"),
        "stdout must contain the CLAUDE_CODE_CONTEXT header; got:\n{stdout}"
    );
    let written = std::fs::read_to_string(standing_dir.join("CLAUDE_CODE_CONTEXT.md"))
        .expect("CLAUDE_CODE_CONTEXT.md must be written next to standing.json");
    assert!(
        written.contains("# CLAUDE_CODE_CONTEXT"),
        "written file must carry the header; got:\n{written}"
    );
}
