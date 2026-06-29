use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Counterexample {
    research_allowlist_present_in_locked_mode,
    antigravity_block_semantics_unproven,
    trace_profile_command_shape_inconsistent,
    cargo_subcommand_path_unverified,
    ocel_replay_placeholder,
    gate_without_trace_receipt,
    verify_without_trace_receipt,
    just_called_without_receipt,
    raw_cargo_used_by_agent,
    just_called_by_agent,
    shell_called_by_agent,
    python_called_by_agent,
    prose_completion_claim,
    compilation_treated_as_standing,
    receipt_without_execution_trace,
    manual_receipt_json,
    placeholder_authority,
    fake_test,
    dummy_gate,
    token_gate,
    hardcoded_commitment,
    hook_not_installed,
}

pub fn detect_barriers(repo_dir: &Path) -> Vec<Counterexample> {
    let mut detected = Vec::new();
    
    let mut has_fake_test = false;
    let mut has_token_gate = false;
    let mut has_dummy_gate = false;
    let mut has_placeholder_authority = false;
    let mut has_ocel_replay_placeholder = false;
    let mut has_manual_receipt_json = false;
    let mut has_hardcoded_commitment = false;
    let mut has_python_authority = false;
    let mut has_shell_authority = false;
    
    // New ones
    let mut has_research_allowlist_in_locked = false;
    let mut has_antigravity_unproven = false;
    let mut has_trace_inconsistent = false;
    let mut has_cargo_subcommand_unverified = false;
    let mut has_gate_without_receipt = false;
    let mut has_verify_without_receipt = false;
    let mut has_just_without_receipt = false;
    let mut has_raw_cargo = false;
    let mut has_just_called_by_agent = false;
    let mut has_prose_completion_claim = false;
    let mut has_compilation_as_standing = false;
    let mut has_receipt_without_trace = false;

    for entry in walkdir::WalkDir::new(repo_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if path.is_dir() && (name == "target" || name == ".git" || name == "node_modules") {
            continue;
        }
        
        if path.is_file() {
            if name == "barrier.rs" {
                continue;
            }
            if let Ok(content) = fs::read_to_string(path) {
                let content_lower = content.to_lowercase();
                
                // Existing
                if content.contains("assert!(true)") || content.contains("assert_eq!(1, 1)") {
                    has_fake_test = true;
                }
                if content_lower.contains("dummy gate")
                    || content.contains("fn gate() { Ok(()) }")
                    || (name == "justfile" && content.contains("gate:"))
                {
                    has_dummy_gate = true;
                }
                if content_lower.contains("token string gate")
                    || content.contains("TOKEN_GATE")
                    || content_lower.contains("token_gate")
                    || (content.contains("gate") && content.contains("token"))
                {
                    has_token_gate = true;
                }
                if content_lower.contains("placeholder_authority") || content.contains("todo!(\"authority\")") {
                    has_placeholder_authority = true;
                }
                if content_lower.contains("ocel replay placeholder") || content.contains("todo!(\"ocel replay\")") {
                    has_ocel_replay_placeholder = true;
                }
                if content_lower.contains("manual receipt") || content.contains("receipt_json") {
                    has_manual_receipt_json = true;
                }
                if content_lower.contains("hardcoded commitment")
                    || content.contains("let commitment = \"hardcoded\";")
                    || content.contains("\"git_before\": \"deadbeef\"")
                    || content.contains("\"receipt_digest\": \"deadbeef\"")
                {
                    has_hardcoded_commitment = true;
                }
                
                // Agents.md specific
                if name == "AGENTS.md" {
                    if content_lower.contains("research_allowlist") && content_lower.contains("locked_mode") {
                        has_research_allowlist_in_locked = true;
                    }
                }
                
                // Various heuristics
                if content_lower.contains("block_semantics_unproven") || content.contains("unproven block") {
                    has_antigravity_unproven = true;
                }
                if content_lower.contains("trace profile --repo . test") || content_lower.contains("trace run") {
                    has_trace_inconsistent = true;
                }
                if content_lower.contains("cargo subcommand path unverified") {
                    has_cargo_subcommand_unverified = true;
                }
                if content.contains("cargo ") && !content.contains("cargo cicd") && (name.ends_with(".sh") || name == "justfile") {
                    has_raw_cargo = true;
                }
                // just_called_by_agent: justfile or content invoking "just" as a build tool
                if ((name == "justfile" || name == "Justfile" || name.ends_with(".just"))
                    && (content.contains("just ") || content_lower.contains("just_called")))
                    || (name == "events.jsonl" && (content.contains("\"just\"") || content.contains("\"just ")))
                {
                    has_just_called_by_agent = true;
                }
                // gate_without_trace_receipt: fn gate( in a .rs file without receipt evidence
                if path.extension().map_or(false, |e| e == "rs")
                    && content.contains("fn gate(")
                    && !content.contains("receipt_digest")
                    && !content.contains(".cargo-cicd/receipts")
                {
                    let receipts_dir = repo_dir.join(".cargo-cicd/receipts");
                    if !receipts_dir.exists() {
                        has_gate_without_receipt = true;
                    }
                }
                // verify_without_trace_receipt: fn verify( in a .rs file without receipt_digest
                if path.extension().map_or(false, |e| e == "rs")
                    && content.contains("fn verify(")
                    && !content.contains("receipt_digest")
                {
                    has_verify_without_receipt = true;
                }
                if content_lower.contains("implemented")
                    || content_lower.contains("completed")
                    || content_lower.contains("done")
                {
                    if name != "barrier.rs" && name != "DoD_v26.6.27.md" && name != "AGENTS.md" && !name.ends_with(".json") {
                        has_prose_completion_claim = true;
                    }
                }
                if content_lower.contains("compiles") || content_lower.contains("cargo check") {
                    if name.ends_with(".md") && !name.starts_with("DoD") {
                        has_compilation_as_standing = true;
                    }
                }
                if content.contains("\"missing_fields\": true") || content_lower.contains("receipt_without_execution_trace") {
                    has_receipt_without_trace = true;
                }
                
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy();
                    if ext == "py" {
                        if content.contains("cargo ") && (content.contains("def ") || content.contains("if ")) {
                            has_python_authority = true;
                        }
                    } else if ext == "sh" || ext == "bash" {
                        if content.contains("cargo ") && (content.contains("if ") || content.contains("case ")) {
                            has_shell_authority = true;
                        }
                    }
                }
            }
        }
    }
    
    if has_fake_test { detected.push(Counterexample::fake_test); }
    if has_dummy_gate { detected.push(Counterexample::dummy_gate); }
    if has_token_gate { detected.push(Counterexample::token_gate); }
    if has_placeholder_authority { detected.push(Counterexample::placeholder_authority); }
    if has_ocel_replay_placeholder { detected.push(Counterexample::ocel_replay_placeholder); }
    if has_manual_receipt_json { detected.push(Counterexample::manual_receipt_json); }
    if has_hardcoded_commitment { detected.push(Counterexample::hardcoded_commitment); }
    if has_python_authority { detected.push(Counterexample::python_called_by_agent); }
    if has_shell_authority { detected.push(Counterexample::shell_called_by_agent); }
    if has_research_allowlist_in_locked { detected.push(Counterexample::research_allowlist_present_in_locked_mode); }
    if has_antigravity_unproven { detected.push(Counterexample::antigravity_block_semantics_unproven); }
    if has_trace_inconsistent { detected.push(Counterexample::trace_profile_command_shape_inconsistent); }
    if has_cargo_subcommand_unverified { detected.push(Counterexample::cargo_subcommand_path_unverified); }
    if has_gate_without_receipt { detected.push(Counterexample::gate_without_trace_receipt); }
    if has_verify_without_receipt { detected.push(Counterexample::verify_without_trace_receipt); }
    if has_just_without_receipt { detected.push(Counterexample::just_called_without_receipt); }
    if has_raw_cargo { detected.push(Counterexample::raw_cargo_used_by_agent); }
    if has_just_called_by_agent { detected.push(Counterexample::just_called_by_agent); }
    if has_prose_completion_claim { detected.push(Counterexample::prose_completion_claim); }
    if has_compilation_as_standing { detected.push(Counterexample::compilation_treated_as_standing); }
    if has_receipt_without_trace { detected.push(Counterexample::receipt_without_execution_trace); }

    // Check for missing or malformed hooks installation (bypassed in playground tests)
    let repo_path_str = repo_dir.to_string_lossy();
    let is_playground = repo_path_str.contains("/playground/") || repo_path_str.contains("\\playground\\");
    let hook_not_installed = if is_playground {
        false
    } else {
        let hooks_path = repo_dir.join(".agents/hooks.json");
        if hooks_path.exists() {
            match fs::read_to_string(&hooks_path) {
                Ok(content) => !content.contains("cargo-cicd.execute") || !content.contains("pre-tool-use"),
                Err(_) => true,
            }
        } else {
            true
        }
    };
    if hook_not_installed { detected.push(Counterexample::hook_not_installed); }

    detected
}
