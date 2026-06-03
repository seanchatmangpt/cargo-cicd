//! protocol_initialize — proof tests for LSP server capabilities declaration.
//!
//! Verifies that build_server_capabilities() declares the required LSP
//! capabilities for editor diagnostics integration (Law 5 proof).

use cargo_cicd_lsp::server::capabilities::build_server_capabilities;

/// build_server_capabilities() must declare diagnosticProvider (Some).
/// This is the capability that enables push-based editor diagnostics.
#[test]
fn build_server_capabilities_declares_diagnostic_provider() {
    let caps = build_server_capabilities();
    assert!(
        caps.diagnostic_provider.is_some(),
        "diagnosticProvider must be declared in ServerCapabilities (got None)"
    );
}

/// build_server_capabilities() must declare code action support.
#[test]
fn build_server_capabilities_declares_code_action_provider() {
    let caps = build_server_capabilities();
    assert!(
        caps.code_action_provider.is_some(),
        "codeActionProvider must be declared in ServerCapabilities (got None)"
    );
}

/// build_server_capabilities() must declare text document sync.
/// Without this editors will not send document-open/change notifications.
#[test]
fn build_server_capabilities_declares_text_document_sync() {
    let caps = build_server_capabilities();
    assert!(
        caps.text_document_sync.is_some(),
        "textDocumentSync must be declared in ServerCapabilities (got None)"
    );
}

/// All three required capabilities must be declared together — partial
/// capability declaration is not acceptable.
#[test]
fn build_server_capabilities_all_required_present() {
    let caps = build_server_capabilities();
    let missing: Vec<&str> = [
        ("diagnosticProvider", caps.diagnostic_provider.is_some()),
        ("codeActionProvider", caps.code_action_provider.is_some()),
        ("textDocumentSync", caps.text_document_sync.is_some()),
    ]
    .iter()
    .filter_map(|(name, present)| if !present { Some(*name) } else { None })
    .collect();

    assert!(
        missing.is_empty(),
        "required LSP capabilities missing: {:?}",
        missing
    );
}
