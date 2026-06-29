//! Code provenance tracking for Vision 2030 process conformance.
//!
//! Git co-author parsing: [`detect_provenance_from_git`] reads the last 20
//! commits and checks `Co-Authored-By` trailers for known AI tool names.
//!
//! Vision 2030 requires that all code is tagged with its authorship origin:
//! human-authored, AI-assisted (human-led with AI suggestions), or AI-generated
//! (AI-led with human review/approval). This metadata is embedded in
//! `ProcessEvent` XES evidence traces so the wasm4pm oracle can enforce
//! provenance-aware adjudication policies.
//!
//! ## How to declare provenance
//!
//! Set the `CICD_CODE_PROVENANCE` environment variable before running
//! cargo-cicd:
//!
//! ```sh
//! # Human-authored code
//! export CICD_CODE_PROVENANCE=human
//!
//! # AI-assisted (e.g. with GitHub Copilot)
//! export CICD_CODE_PROVENANCE=ai-assisted:copilot
//!
//! # AI-generated (e.g. with Claude)
//! export CICD_CODE_PROVENANCE=ai-generated:claude
//! ```
//!
//! If unset, cargo-cicd uses heuristic detection on the source files touched
//! in the current git diff to infer likely provenance.

/// Classification of code authorship origin.
#[derive(Debug, Clone, PartialEq)]
pub enum CodeProvenance {
    /// Entirely human-authored. No AI tooling was used in writing this code.
    Human,
    /// Human-authored with AI assistance: AI made suggestions, a human reviewed
    /// and accepted or modified them before committing.
    AiAssisted {
        /// Name of the AI tool used (e.g. `"copilot"`, `"claude"`, `"codeium"`).
        tool: String,
    },
    /// Primarily AI-generated: the AI wrote most of the code, a human reviewed
    /// and approved the output before committing.
    AiGenerated {
        /// Name of the AI tool used (e.g. `"claude"`, `"gpt-4"`, `"gemini"`).
        tool: String,
    },
    /// Provenance was not declared and could not be inferred.
    Unknown,
}

impl CodeProvenance {
    /// Serialize to a short tag string for embedding in XES trace attributes.
    ///
    /// | Variant | Tag |
    /// |---------|-----|
    /// | `Human` | `"human"` |
    /// | `AiAssisted { tool }` | `"ai-assisted"` (tool name in separate attribute) |
    /// | `AiGenerated { tool }` | `"ai-generated"` (tool name in separate attribute) |
    /// | `Unknown` | `"unknown"` |
    pub fn to_tag(&self) -> &'static str {
        match self {
            CodeProvenance::Human => "human",
            CodeProvenance::AiAssisted { .. } => "ai-assisted",
            CodeProvenance::AiGenerated { .. } => "ai-generated",
            CodeProvenance::Unknown => "unknown",
        }
    }

    /// Parse a tag string (as produced by [`to_tag`] or `CICD_CODE_PROVENANCE`) back
    /// into a `CodeProvenance`.
    ///
    /// Formats accepted:
    /// - `"human"` → `Human`
    /// - `"ai-assisted"` → `AiAssisted { tool: "unknown" }`
    /// - `"ai-assisted:copilot"` → `AiAssisted { tool: "copilot" }`
    /// - `"ai-generated"` → `AiGenerated { tool: "unknown" }`
    /// - `"ai-generated:claude"` → `AiGenerated { tool: "claude" }`
    /// - anything else → `Unknown`
    pub fn from_tag(tag: &str) -> Self {
        let tag = tag.trim();
        if tag == "human" {
            return CodeProvenance::Human;
        }
        if tag.starts_with("ai-assisted") {
            let tool = tag
                .strip_prefix("ai-assisted:")
                .unwrap_or("unknown")
                .to_string();
            return CodeProvenance::AiAssisted { tool };
        }
        if tag.starts_with("ai-generated") {
            let tool = tag
                .strip_prefix("ai-generated:")
                .unwrap_or("unknown")
                .to_string();
            return CodeProvenance::AiGenerated { tool };
        }
        CodeProvenance::Unknown
    }

    /// Return the tool name, if any.
    pub fn tool_name(&self) -> Option<&str> {
        match self {
            CodeProvenance::AiAssisted { tool } | CodeProvenance::AiGenerated { tool } => {
                Some(tool.as_str())
            }
            _ => None,
        }
    }
}

/// Known AI tool name fragments (lower-case) for co-author detection.
const AI_TOOLS: &[&str] = &["claude", "copilot", "gpt", "gemini", "codeium"];

/// Detect code provenance by inspecting git log co-author trailers.
///
/// Runs `git log --format="%aN <%ae>%n%(trailers:key=Co-Authored-By,valueonly)" -20`
/// in `repo_dir` and searches `Co-Authored-By` lines for known AI tool names.
///
/// Returns:
/// - [`CodeProvenance::AiAssisted`] with the first matched tool name, or
/// - [`CodeProvenance::Unknown`] if git fails, is not a repo, or no AI tool
///   name is found.
///
/// Never panics.
pub fn detect_provenance_from_git(repo_dir: &std::path::Path) -> CodeProvenance {
    let output = match std::process::Command::new("git")
        .args(&[
            "log",
            "--format=%aN <%ae>%n%(trailers:key=Co-Authored-By,valueonly)",
            "-20",
        ])
        .current_dir(repo_dir)
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return CodeProvenance::Unknown,
    };

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let lower = line.to_lowercase();
        for &tool in AI_TOOLS {
            if lower.contains(tool) {
                return CodeProvenance::AiAssisted {
                    tool: tool.to_string(),
                };
            }
        }
    }
    CodeProvenance::Unknown
}

/// Emit XES-style provenance attributes for embedding in a trace event.
///
/// Returns a map with:
/// - `"provenance:tag"` — e.g. `"ai-assisted"`
/// - `"provenance:tool"` — tool name or `""` if not applicable
/// - `"provenance:detection_method"` — always `"git-log"`
pub fn emit_provenance_xes_attributes(
    provenance: &CodeProvenance,
) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    map.insert("provenance:tag".to_string(), provenance.to_tag().to_string());
    map.insert(
        "provenance:tool".to_string(),
        provenance.tool_name().unwrap_or("").to_string(),
    );
    map.insert(
        "provenance:detection_method".to_string(),
        "git-log".to_string(),
    );
    map
}

/// Patterns commonly seen in LLM-generated Rust source code.
///
/// Each entry is `(pattern, confidence_weight)` where the weight represents
/// how strongly this pattern suggests LLM authorship (0.0–1.0 per occurrence,
/// capped at 1.0 total).
const LLM_SIGNALS: &[(&str, f32)] = &[
    ("// This function", 0.10),
    ("// The following", 0.10),
    ("// Implementation of", 0.15),
    ("unwrap_or_else(|e| panic!", 0.20),
    ("// TODO: implement", 0.05),
    ("// Note:", 0.05),
    ("// Helper function", 0.10),
    ("/// # Examples", 0.05),
    ("/// # Panics", 0.05),
    ("/// # Errors", 0.05),
    // LLMs often add SAFETY comments even where the code is entirely safe.
    ("// SAFETY:", 0.05),
    ("let result = ", 0.05),
    // Over-verbose match arms with redundant comments are a common LLM tell.
    ("// Handle the case where", 0.10),
    ("// This is because", 0.10),
];

/// A single LLM heuristic signal found in source code.
#[derive(Debug)]
pub struct LlmSignal {
    /// The pattern string that matched.
    pub pattern: &'static str,
    /// 1-based line number of the match.
    pub line: usize,
    /// Confidence contribution from this signal (0.0–1.0).
    pub weight: f32,
}

/// Result of heuristic LLM code detection on a Rust source string.
#[derive(Debug)]
pub struct LlmDetectionResult {
    /// Overall confidence that this code is LLM-generated.
    ///
    /// - `0.0` → almost certainly human-authored.
    /// - `1.0` → almost certainly LLM-generated.
    ///
    /// This is the sum of all signal weights, clamped to `[0.0, 1.0]`.
    pub confidence: f32,
    /// Individual pattern matches that contributed to the confidence score.
    pub signals: Vec<LlmSignal>,
}

/// Run heuristic LLM detection on a Rust source string.
///
/// Scans every line for patterns listed in [`LLM_SIGNALS`] and accumulates
/// a confidence score. The score is the sum of all matched signal weights,
/// clamped to `[0.0, 1.0]`.
///
/// # Limitations
///
/// - False positives: well-documented human code may match some signals.
/// - False negatives: minimally commented LLM code may score near zero.
/// - This is a heuristic, not a deterministic classifier.
pub fn detect_llm_patterns(source: &str) -> LlmDetectionResult {
    let mut signals = Vec::new();
    let mut total_weight = 0.0f32;

    for (line_num, line) in source.lines().enumerate() {
        for &(pattern, weight) in LLM_SIGNALS {
            if line.contains(pattern) {
                signals.push(LlmSignal {
                    pattern,
                    line: line_num + 1, // 1-based
                    weight,
                });
                total_weight += weight;
            }
        }
    }

    LlmDetectionResult {
        confidence: total_weight.clamp(0.0, 1.0),
        signals,
    }
}

/// A summary of provenance across multiple source files.
#[derive(Debug)]
pub struct ProvenanceSummary {
    /// Combined provenance tag for the overall submission.
    ///
    /// Derived from `CICD_CODE_PROVENANCE` env var if set; otherwise inferred
    /// from heuristic scan results.
    pub tag: String,
    /// Number of files that were scanned.
    pub files_scanned: usize,
    /// Number of files where `detect_llm_patterns` returned confidence > 0.5.
    pub likely_llm_files: usize,
    /// Average LLM confidence across all scanned files (0.0 if no files scanned).
    pub avg_confidence: f32,
}

/// Build a provenance summary by scanning `file_paths`.
///
/// If `CICD_CODE_PROVENANCE` is set in the environment, it takes precedence
/// over heuristic detection and is used as the `tag`. Otherwise the tag is
/// derived from the heuristic scan:
///
/// - avg_confidence ≥ 0.5 → `"ai-generated:unknown"`
/// - avg_confidence ≥ 0.2 → `"ai-assisted:unknown"`
/// - avg_confidence < 0.2 → `"human"`
///
/// Files that cannot be read are silently skipped.
pub fn summarize_provenance(file_paths: &[String]) -> ProvenanceSummary {
    // 1. Env override takes precedence.
    let env_tag = std::env::var("CICD_CODE_PROVENANCE")
        .ok()
        .filter(|v| !v.trim().is_empty());

    if let Some(ref tag) = env_tag {
        if file_paths.is_empty() {
            return ProvenanceSummary {
                tag: tag.clone(),
                files_scanned: 0,
                likely_llm_files: 0,
                avg_confidence: 0.0,
            };
        }
    }

    if file_paths.is_empty() {
        // 2. No files — try git from cwd, then fall back to unknown.
        let git_provenance = detect_provenance_from_git(std::path::Path::new("."));
        let tag = env_tag.unwrap_or_else(|| match &git_provenance {
            CodeProvenance::Unknown => "unknown".to_string(),
            p => {
                let base = p.to_tag();
                if let Some(tool) = p.tool_name() {
                    format!("{}:{}", base, tool)
                } else {
                    base.to_string()
                }
            }
        });
        return ProvenanceSummary {
            tag,
            files_scanned: 0,
            likely_llm_files: 0,
            avg_confidence: 0.0,
        };
    }

    // 3. Primary signal: git co-author detection from cwd.
    let git_provenance = detect_provenance_from_git(std::path::Path::new("."));

    // 4. Secondary signal: content heuristics.
    let mut total_confidence = 0.0f32;
    let mut likely_llm_files = 0usize;
    let mut files_scanned = 0usize;

    for path in file_paths {
        let content = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue, // Silently skip unreadable files.
        };
        files_scanned += 1;
        let result = detect_llm_patterns(&content);
        total_confidence += result.confidence;
        if result.confidence > 0.5 {
            likely_llm_files += 1;
        }
    }

    let avg_confidence = if files_scanned > 0 {
        total_confidence / files_scanned as f32
    } else {
        0.0
    };

    // Prefer git result over content heuristics.
    let inferred_tag = match &git_provenance {
        CodeProvenance::Unknown => {
            // Fall back to content heuristics.
            if avg_confidence >= 0.5 {
                "ai-generated:unknown".to_string()
            } else if avg_confidence >= 0.2 {
                "ai-assisted:unknown".to_string()
            } else {
                "human".to_string()
            }
        }
        p => {
            let base = p.to_tag();
            if let Some(tool) = p.tool_name() {
                format!("{}:{}", base, tool)
            } else {
                base.to_string()
            }
        }
    };

    ProvenanceSummary {
        tag: env_tag.unwrap_or(inferred_tag),
        files_scanned,
        likely_llm_files,
        avg_confidence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_llm_patterns_low_confidence_for_clean_code() {
        let source = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    println!("{}", add(1, 2));
}
"#;
        let result = detect_llm_patterns(source);
        assert!(
            result.confidence < 0.3,
            "clean code should have low LLM confidence, got {}",
            result.confidence
        );
    }

    #[test]
    fn detect_llm_patterns_high_confidence_for_llm_style() {
        // Deliberately heavy with LLM-style comments.
        let source = r#"
// This function computes the sum of two numbers.
// Implementation of the add operation.
// Note: this is a simple helper function.
// Helper function for arithmetic operations.
// The following code handles the calculation.
fn add(a: i32, b: i32) -> i32 {
    // This is because we need to return the sum.
    let result = a + b;
    result
}
"#;
        let result = detect_llm_patterns(source);
        assert!(
            result.confidence >= 0.3,
            "LLM-style code should have higher confidence, got {}",
            result.confidence
        );
    }

    #[test]
    fn detect_llm_patterns_finds_this_function_pattern() {
        let source = "// This function initializes the workspace.\nfn init() {}";
        let result = detect_llm_patterns(source);
        let found = result
            .signals
            .iter()
            .any(|s| s.pattern == "// This function");
        assert!(found, "expected to find '// This function' signal");
    }

    #[test]
    fn git_provenance_unknown_when_not_git_repo() {
        let result = detect_provenance_from_git(std::path::Path::new("/tmp"));
        assert_eq!(result, CodeProvenance::Unknown);
    }

    #[test]
    fn provenance_env_override_takes_precedence() {
        std::env::set_var("CICD_CODE_PROVENANCE", "ai-generated:claude");
        let summary = summarize_provenance(&[]);
        std::env::remove_var("CICD_CODE_PROVENANCE");
        assert_eq!(summary.tag, "ai-generated:claude");
    }
}
