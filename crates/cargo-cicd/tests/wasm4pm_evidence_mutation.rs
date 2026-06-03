//! wasm4pm evidence gate — mutation (negative) cases.
//! Each test emits valid XES, then mutates it, then asserts wasm4pm refuses.
//! Proves wasm4pm is a real adjudicator, not a rubber stamp.

use cargo_cicd::evidence::{
    assert_wpm_verdict, emit_xes, ExpectedWpmVerdict, ProcessEvent, WpmEvidenceOracle,
};
use tempfile::TempDir;

/// Corrupted XML (not valid XML at all) must be refused.
#[test]
fn evidence_mutation_corrupted_xes_refused() {
    let dir = TempDir::new().unwrap();
    let xes_path = dir.path().join("mutated.xes");
    // Write intentionally malformed XES
    std::fs::write(&xes_path, "NOT VALID XML AT ALL").unwrap();
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Refuse);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

/// Empty file must be refused — no evidence is not acceptance.
#[test]
fn evidence_mutation_empty_xes_refused() {
    let dir = TempDir::new().unwrap();
    let xes_path = dir.path().join("empty.xes");
    std::fs::write(&xes_path, b"").unwrap();
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Refuse);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

/// XES with a mismatched closing tag is invalid XML — the parser must reject it.
/// </wrong_close> is structurally impossible: no matching open tag exists.
#[test]
fn evidence_mutation_mismatched_tags_refused() {
    let dir = TempDir::new().unwrap();
    let xes_path = dir.path().join("mismatched.xes");
    let events = vec![ProcessEvent::new("status:show", "PASS")];
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    corrupt_xes_mismatched_tags(&xes_path);
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Refuse);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

/// Binary garbage must be refused.
#[test]
fn evidence_mutation_binary_garbage_refused() {
    let dir = TempDir::new().unwrap();
    let xes_path = dir.path().join("garbage.xes");
    std::fs::write(&xes_path, b"\x00\x01\x02\xff\xfe NOT XML").unwrap();
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Refuse);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

/// Truncated XES (mid-element) must be refused.
#[test]
fn evidence_mutation_truncated_xes_refused() {
    let dir = TempDir::new().unwrap();
    let xes_path = dir.path().join("truncated.xes");
    let events = vec![ProcessEvent::new("status:show", "PASS")];
    emit_xes(&events, &xes_path).expect("emit_xes must not fail");
    // Truncate to first 20 bytes — cuts off mid-element
    let content = std::fs::read(&xes_path).unwrap();
    std::fs::write(&xes_path, &content[..20.min(content.len())]).unwrap();
    let oracle = WpmEvidenceOracle::new();
    if oracle.is_available() {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Refuse);
    } else {
        assert_wpm_verdict(&oracle, &xes_path, &ExpectedWpmVerdict::Blocked);
    }
}

// ── Mutation helper functions ────────────────────────────────────────────────
// These are exported pub so that wasm4pm_refusal_cases.rs can use them via
// `mod wasm4pm_evidence_mutation; use wasm4pm_evidence_mutation::*;`.
// Each function mutates an existing XES file at `path` in a way that should
// cause wasm4pm to refuse the evidence.

use std::path::Path;

/// Replace the first closing `</event>` tag with `</wrong_close>`, creating
/// a mismatched-tag error that a conforming XML parser must reject at exit 1.
pub fn corrupt_xes_mismatched_tags(path: &Path) {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mutated = content.replacen("</event>", "</wrong_close>", 1);
    std::fs::write(path, mutated).unwrap();
}

/// Replace a verdict attribute value with a contradictory one (e.g. "pass" → "FAIL"),
/// making the XES semantically inconsistent.
pub fn corrupt_xes_contradictory_verdict(path: &Path) {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mutated = content.replace("pass", "FAIL").replace("PASS", "FAIL");
    std::fs::write(path, mutated).unwrap();
}

/// Remove the `<trace>` element from the XES so there is no process evidence.
pub fn corrupt_xes_missing_trace(path: &Path) {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    // Strip everything between <trace> and </trace> including the tags
    let start = content.find("<trace>").unwrap_or(content.len());
    let end = content
        .find("</trace>")
        .map(|i| i + "</trace>".len())
        .unwrap_or(content.len());
    let mutated = format!("{}{}", &content[..start], &content[end..]);
    std::fs::write(path, mutated).unwrap();
}

/// Remove the closing `</log>` tag, producing malformed XML.
pub fn corrupt_xes_no_closing_tag(path: &Path) {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mutated = content.replace("</log>", "");
    std::fs::write(path, mutated).unwrap();
}

/// Overwrite the file with empty content — no evidence is not acceptance.
pub fn corrupt_xes_empty_file(path: &Path) {
    std::fs::write(path, b"").unwrap();
}

/// Overwrite the file with binary garbage — cannot be parsed as XES.
pub fn corrupt_xes_binary_garbage(path: &Path) {
    std::fs::write(path, b"\x00\x01\x02\xff\xfe \xde\xad\xbe\xef NOT XML").unwrap();
}

/// Truncate the file to 20 bytes, cutting off mid-element.
pub fn corrupt_xes_truncated(path: &Path) {
    let content = std::fs::read(path).unwrap_or_default();
    std::fs::write(path, &content[..20.min(content.len())]).unwrap();
}

/// Replace a valid attribute value with an XML-invalid character sequence.
pub fn corrupt_xes_invalid_attribute(path: &Path) {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    // Inject an unescaped `<` inside an attribute value — invalid XML
    let mutated = content.replacen("value=\"pass\"", "value=\"<invalid&>\"", 1);
    // If "pass" wasn't found, inject garbage somewhere else
    let mutated = if mutated == content {
        content.replacen("concept:name", "concept:name value=\"<bad\"", 1)
    } else {
        mutated
    };
    std::fs::write(path, mutated).unwrap();
}

/// Replace the XML encoding declaration with a non-UTF-8 encoding that
/// conflicts with the actual UTF-8 file content.
pub fn corrupt_xes_wrong_encoding_declaration(path: &Path) {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mutated = content.replace(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
        "<?xml version=\"1.0\" encoding=\"EBCDIC-US\"?>",
    );
    // If the declaration was absent, prepend a conflicting one
    let mutated = if mutated == content {
        format!(
            "<?xml version=\"1.0\" encoding=\"EBCDIC-US\"?>\n{}",
            content
        )
    } else {
        mutated
    };
    std::fs::write(path, mutated).unwrap();
}
