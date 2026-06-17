//! Tests for code provenance detection and classification.
//!
//! Covers:
//! - `detect_llm_patterns` confidence scores for clean vs. LLM-style code
//! - `CodeProvenance::to_tag` / `from_tag` roundtrips
//! - `AiAssisted` and `AiGenerated` serialization with tool names
//! - `summarize_provenance` with empty input
//! - `LlmDetectionResult` correct line numbers in signals
//! - Pattern detection for specific LLM-style markers

use cargo_cicd::code_provenance::{
    detect_llm_patterns, summarize_provenance, CodeProvenance,
};

// ── detect_llm_patterns tests ─────────────────────────────────────────────────

/// 1. Clean human-authored code returns low LLM confidence.
#[test]
fn detect_llm_patterns_low_confidence_for_clean_code() {
    let source = r#"
use std::collections::HashMap;

pub struct Registry {
    entries: HashMap<String, u32>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: u32) {
        self.entries.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<u32> {
        self.entries.get(key).copied()
    }
}
"#;
    let result = detect_llm_patterns(source);
    assert!(
        result.confidence < 0.3,
        "clean human code should score < 0.3, got {}",
        result.confidence
    );
}

/// 2. Heavily commented LLM-style code returns high confidence.
#[test]
fn detect_llm_patterns_high_confidence_for_llm_style_code() {
    let source = r#"
// This function initializes the registry with default values.
// Implementation of the registry pattern for cargo-cicd.
// Note: this is a helper function used by multiple modules.
// Helper function for registry initialization.
// The following code sets up the data structure.
pub fn init_registry() -> Vec<String> {
    // This is because we need a clean starting state.
    let result = Vec::new();
    result
}

// This function processes entries in the registry.
// Implementation of the entry processor.
/// # Errors
/// Returns an error if processing fails.
/// # Panics
/// Panics if the input is empty.
/// # Examples
/// ```
/// let _ = process_entry("");
/// ```
pub fn process_entry(entry: &str) -> Result<String, String> {
    // Note: the entry must be non-empty for this to work.
    let result = entry.to_uppercase();
    // This is because uppercase is the canonical form.
    Ok(result)
}
"#;
    let result = detect_llm_patterns(source);
    assert!(
        result.confidence >= 0.3,
        "LLM-style code should score >= 0.3, got {}",
        result.confidence
    );
}

/// 7. `detect_llm_patterns` finds the "// This function" pattern.
#[test]
fn detect_llm_patterns_finds_this_function_pattern() {
    let source = "// This function computes the workspace hash.\nfn compute_hash() -> u64 { 0 }";
    let result = detect_llm_patterns(source);
    let found = result
        .signals
        .iter()
        .any(|s| s.pattern == "// This function");
    assert!(
        found,
        "expected to detect '// This function' signal in source"
    );
}

// ── CodeProvenance tag roundtrip tests ───────────────────────────────────────

/// 3. `CodeProvenance::to_tag` roundtrips for all variants via `from_tag`.
#[test]
fn code_provenance_human_to_tag_roundtrips() {
    let p = CodeProvenance::Human;
    let tag = p.to_tag();
    let parsed = CodeProvenance::from_tag(tag);
    assert_eq!(parsed, CodeProvenance::Human, "Human must roundtrip via to_tag/from_tag");
}

#[test]
fn code_provenance_unknown_to_tag_roundtrips() {
    let p = CodeProvenance::Unknown;
    let tag = p.to_tag();
    let parsed = CodeProvenance::from_tag(tag);
    assert_eq!(parsed, CodeProvenance::Unknown, "Unknown must roundtrip via to_tag/from_tag");
}

#[test]
fn code_provenance_ai_generated_to_tag_roundtrips() {
    // Note: to_tag() returns the base tag without tool name.
    // from_tag() with just "ai-generated" uses "unknown" as tool name.
    let p = CodeProvenance::AiGenerated {
        tool: "unknown".to_string(),
    };
    let tag = p.to_tag(); // "ai-generated"
    let parsed = CodeProvenance::from_tag(tag);
    assert_eq!(
        parsed,
        CodeProvenance::AiGenerated {
            tool: "unknown".to_string()
        }
    );
}

/// 4. `AiAssisted { tool }` serializes with the tool name in the env-var format.
#[test]
fn code_provenance_ai_assisted_preserves_tool_name_on_parse() {
    let env_value = "ai-assisted:copilot";
    let parsed = CodeProvenance::from_tag(env_value);
    assert_eq!(
        parsed,
        CodeProvenance::AiAssisted {
            tool: "copilot".to_string()
        },
        "from_tag must extract tool name from colon-separated format"
    );
    assert_eq!(parsed.to_tag(), "ai-assisted");
    assert_eq!(parsed.tool_name(), Some("copilot"));
}

/// 4b. `AiGenerated { tool }` preserves tool name on parse.
#[test]
fn code_provenance_ai_generated_preserves_tool_name_on_parse() {
    let env_value = "ai-generated:claude";
    let parsed = CodeProvenance::from_tag(env_value);
    assert_eq!(
        parsed,
        CodeProvenance::AiGenerated {
            tool: "claude".to_string()
        }
    );
    assert_eq!(parsed.tool_name(), Some("claude"));
}

/// Unknown strings must parse to `Unknown`.
#[test]
fn code_provenance_unknown_for_unrecognized_tag() {
    for tag in &["", "robot", "llm-gen", "partially-human", "AI-ASSISTED"] {
        let p = CodeProvenance::from_tag(tag);
        assert_eq!(
            p,
            CodeProvenance::Unknown,
            "unrecognized tag {:?} should parse to Unknown",
            tag
        );
    }
}

// ── summarize_provenance tests ────────────────────────────────────────────────

/// 5. `summarize_provenance` returns zero counts for empty input.
#[test]
fn summarize_provenance_empty_returns_zeros() {
    let summary = summarize_provenance(&[]);
    assert_eq!(summary.files_scanned, 0, "files_scanned must be 0 for empty input");
    assert_eq!(summary.likely_llm_files, 0, "likely_llm_files must be 0 for empty input");
    assert_eq!(
        summary.avg_confidence, 0.0,
        "avg_confidence must be 0.0 for empty input"
    );
}

/// `summarize_provenance` with unreadable paths silently skips them.
#[test]
fn summarize_provenance_skips_unreadable_files() {
    let paths = vec![
        "/nonexistent/path/to/file1.rs".to_string(),
        "/nonexistent/path/to/file2.rs".to_string(),
    ];
    let summary = summarize_provenance(&paths);
    // Both files are unreadable, so files_scanned stays 0.
    assert_eq!(summary.files_scanned, 0, "unreadable files must be skipped");
}

// ── LlmDetectionResult line number tests ─────────────────────────────────────

/// 6. `LlmDetectionResult` signals report correct 1-based line numbers.
#[test]
fn detect_llm_patterns_correct_line_number_for_signal() {
    // Line 1: blank
    // Line 2: blank
    // Line 3: the pattern
    // Line 4: code
    let source = "\n\n// This function does important work\nfn do_work() {}";
    let result = detect_llm_patterns(source);
    let signal = result
        .signals
        .iter()
        .find(|s| s.pattern == "// This function")
        .expect("'// This function' signal must be present");
    assert_eq!(
        signal.line, 3,
        "signal must report 1-based line number 3, got {}",
        signal.line
    );
}

/// Signals include non-zero weight values.
#[test]
fn detect_llm_patterns_signals_have_positive_weight() {
    let source = "// This function is important.\nfn f() {}";
    let result = detect_llm_patterns(source);
    for signal in &result.signals {
        assert!(signal.weight > 0.0, "all signals must have positive weight");
    }
}

/// Confidence is clamped to [0.0, 1.0] even with many matching patterns.
#[test]
fn detect_llm_patterns_confidence_clamped_to_one() {
    // Flood with every possible pattern to push the sum well over 1.0.
    let source = r#"
// This function implements the core logic.
// The following code handles processing.
// Implementation of the main algorithm.
unwrap_or_else(|e| panic!("error: {}", e))
// TODO: implement this properly
// Note: this is important
// Helper function for setup
/// # Examples
/// # Panics
/// # Errors
// SAFETY: this is technically safe
let result = some_value();
// Handle the case where input is None
// This is because we need to handle edge cases
"#;
    let result = detect_llm_patterns(source);
    assert!(
        result.confidence <= 1.0,
        "confidence must be clamped to <= 1.0, got {}",
        result.confidence
    );
    assert!(
        result.confidence >= 0.0,
        "confidence must be >= 0.0, got {}",
        result.confidence
    );
}

/// `detect_llm_patterns` on an empty string returns zero confidence and no signals.
#[test]
fn detect_llm_patterns_empty_source_returns_zero() {
    let result = detect_llm_patterns("");
    assert_eq!(result.confidence, 0.0, "empty source must score 0.0");
    assert!(result.signals.is_empty(), "empty source must have no signals");
}
