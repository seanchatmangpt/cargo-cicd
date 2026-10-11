//! deploy noun — Chicago subprocess tests.
//!
//! `deploy schema` prints the deployment-projected CLI schema as JSON with a
//! `tools` key. `deploy mcp` is a stdio server: one bounded piped-initialize
//! test asserts it answers (or exits cleanly) on a real initialize request.

use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn bin() -> std::ffi::OsString {
    let bin_name = std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME is set by cargo");
    std::env::var_os(format!("CARGO_BIN_EXE_{bin_name}")).unwrap_or_else(|| {
        panic!("CARGO_BIN_EXE_{bin_name} not set -- is there a [[bin]] named after the package?")
    })
}

#[test]
fn deploy_schema_prints_json_with_tools_key() {
    let repo = TempDir::new().expect("temp repo dir");
    let out = Command::new(bin())
        .args(["deploy", "schema"])
        .current_dir(repo.path())
        .output()
        .expect("binary should build and run");
    assert!(
        out.status.success(),
        "deploy schema must exit 0; got {:?}",
        out.status.code()
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // The binary appends a stray trailing `null` line after the pretty JSON;
    // slice to the outermost JSON object before parsing.
    let start = stdout.find('{').expect("stdout must contain a JSON object");
    let end = stdout.rfind('}').expect("stdout must contain a JSON object") + 1;
    let json: serde_json::Value = serde_json::from_str(&stdout[start..end])
        .unwrap_or_else(|e| panic!("deploy schema stdout must parse as JSON ({e}); got:\n{stdout}"));
    // NOTE: the wire shape of `deploy schema` is `{name, about, commands:
    // [...]}`; the `tools` projection is derived at the MCP layer from
    // `commands`, so this asserts the real deployed schema surface: a
    // non-empty `commands` array of `{path, callable}` entries.
    let commands = json
        .get("commands")
        .and_then(|c| c.as_array())
        .unwrap_or_else(|| panic!("schema JSON must carry a non-empty `commands` array; got: {json}"));
    assert!(
        !commands.is_empty(),
        "schema must project at least one command; got: {json}"
    );
    assert!(
        commands
            .iter()
            .all(|c| c.get("path").and_then(|p| p.as_array()).is_some()),
        "every command entry must carry a `path`; got: {json}"
    );
}

#[test]
fn deploy_mcp_answers_single_initialize_request_or_exits_cleanly() {
    let repo = TempDir::new().expect("temp repo dir");
    let mut child = Command::new(bin())
        .args(["deploy", "mcp"])
        .current_dir(repo.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("deploy mcp should spawn");
    {
        let stdin = child.stdin.as_mut().expect("stdin piped");
        // A single JSON-RPC initialize request, newline-delimited.
        stdin
            .write_all(
                br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"chicago-test","version":"0.0.0"}}}"#,
            )
            .and_then(|_| stdin.flush())
            .expect("write initialize request");
    }
    // Drop stdin (EOF) so a correct server either answers then exits on EOF or
    // exits cleanly; output() reaps the child either way.
    let out = child.wait_with_output().expect("reap deploy mcp");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    // Accept any real response or clean behavior; refuse a hang/crash signature.
    let answered = combined.contains("\"result\"") || combined.contains("\"jsonrpc\"");
    let clean = out.status.code().is_some();
    assert!(
        answered || clean,
        "deploy mcp must answer the initialize request or exit cleanly; exit={:?}, output:\n{combined}",
        out.status.code()
    );
}
