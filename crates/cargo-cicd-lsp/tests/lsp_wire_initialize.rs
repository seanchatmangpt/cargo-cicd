//! lsp_wire_initialize — JSON-RPC wire-level proof that cargo-cicd-lsp
//! starts, responds to an LSP initialize request, and returns capabilities
//! that include diagnosticProvider and textDocumentSync.
//!
//! These tests spawn the real binary over stdio and exercise the tower-lsp
//! JSON-RPC transport — no mocking.  They are #[ignore] by default so they
//! only run when a built binary is available (`cargo test -- --ignored`).

use std::io::{Read, Write};
use std::process::{Command, Stdio};

/// Write one LSP message to `writer` using the Content-Length framing required
/// by the Language Server Protocol specification.
fn send_lsp_message(writer: &mut impl Write, json: &str) {
    let framed = format!("Content-Length: {}\r\n\r\n{}", json.len(), json);
    writer.write_all(framed.as_bytes()).unwrap();
    writer.flush().unwrap();
}

/// Read one LSP message from `reader`, returning the decoded JSON body.
/// Parses the `Content-Length` header then reads exactly that many bytes.
fn read_lsp_response(reader: &mut impl Read) -> serde_json::Value {
    // Accumulate header bytes until we see the \r\n\r\n terminator.
    let mut header = String::new();
    let mut byte = [0u8; 1];
    loop {
        reader
            .read_exact(&mut byte)
            .expect("unexpected EOF reading LSP header");
        header.push(byte[0] as char);
        if header.ends_with("\r\n\r\n") {
            break;
        }
    }

    let content_length: usize = header
        .lines()
        .find(|l| l.starts_with("Content-Length:"))
        .expect("no Content-Length header in LSP response")
        .trim_start_matches("Content-Length:")
        .trim()
        .parse()
        .expect("Content-Length is not a valid usize");

    let mut body = vec![0u8; content_length];
    reader
        .read_exact(&mut body)
        .expect("unexpected EOF reading LSP body");

    serde_json::from_slice(&body).expect("LSP response body is not valid JSON")
}

// ---------------------------------------------------------------------------
// Wire tests (require built binary — run with `cargo test -- --ignored`)
// ---------------------------------------------------------------------------

/// Spawns cargo-cicd-lsp, sends an LSP `initialize` request, and asserts
/// that the response declares `diagnosticProvider` and `textDocumentSync`.
#[test]
#[ignore]
fn lsp_wire_initialize_returns_diagnostic_capabilities() {
    let binary = std::env!("CARGO_BIN_EXE_cargo-cicd-lsp");

    let mut child = Command::new(binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn cargo-cicd-lsp — is the binary built?");

    // Send initialize request.
    {
        let stdin = child.stdin.as_mut().expect("child stdin missing");
        let init_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": null,
                "capabilities": {},
                "rootUri": null
            }
        })
        .to_string();
        send_lsp_message(stdin, &init_req);
    }

    // Read the response on a separate thread so we can impose a timeout.
    let mut stdout = child.stdout.take().expect("child stdout missing");
    let handle = std::thread::spawn(move || read_lsp_response(&mut stdout));

    let response = handle
        .join()
        .expect("reader thread panicked while reading LSP response");

    // Clean up the child process regardless of assertion outcome.
    child.kill().ok();
    child.wait().ok();

    // --- assertions ---

    assert_eq!(
        response["jsonrpc"], "2.0",
        "response must be JSON-RPC 2.0: {}",
        response
    );
    assert_eq!(
        response["id"], 1,
        "response id must match request id: {}",
        response
    );
    assert!(
        response["error"].is_null(),
        "initialize returned an error: {}",
        response
    );

    let caps = &response["result"]["capabilities"];

    // diagnosticProvider — the primary capability this crate exists to prove.
    assert!(
        !caps["diagnosticProvider"].is_null(),
        "diagnosticProvider missing from initialize result.capabilities: {}",
        response
    );

    // textDocumentSync — required for editors to send document notifications.
    assert!(
        !caps["textDocumentSync"].is_null(),
        "textDocumentSync missing from initialize result.capabilities: {}",
        response
    );
}

/// Verifies that the initialize response id echoes the request id correctly
/// for a non-integer id (string id per JSON-RPC spec).
#[test]
#[ignore]
fn lsp_wire_initialize_echoes_string_id() {
    let binary = std::env!("CARGO_BIN_EXE_cargo-cicd-lsp");

    let mut child = Command::new(binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn cargo-cicd-lsp");

    {
        let stdin = child.stdin.as_mut().unwrap();
        let init_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "req-42",
            "method": "initialize",
            "params": {
                "processId": null,
                "capabilities": {},
                "rootUri": null
            }
        })
        .to_string();
        send_lsp_message(stdin, &init_req);
    }

    let mut stdout = child.stdout.take().unwrap();
    let handle = std::thread::spawn(move || read_lsp_response(&mut stdout));
    let response = handle.join().expect("reader thread panicked");
    child.kill().ok();
    child.wait().ok();

    assert_eq!(
        response["id"], "req-42",
        "response id must echo the string request id: {}",
        response
    );
}
