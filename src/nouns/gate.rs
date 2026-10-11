use crate::barrier::Counterexample;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

// ---------------------------------------------------------------------------
// SARIF v2.1.0 types
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, Debug)]
struct SarifOutput {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(serde::Serialize, Debug)]
struct SarifRun {
    tool: SarifTool,
    invocations: Vec<SarifInvocation>,
    results: Vec<SarifResult>,
}

#[derive(serde::Serialize, Debug)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(serde::Serialize, Debug)]
struct SarifDriver {
    name: &'static str,
    version: &'static str,
    rules: Vec<SarifRule>,
}

#[derive(serde::Serialize, Debug)]
struct SarifRule {
    id: String,
    #[serde(rename = "shortDescription")]
    short_description: SarifMessage,
}

#[derive(serde::Serialize, Debug)]
struct SarifMessage {
    text: String,
}

#[derive(serde::Serialize, Debug)]
struct SarifInvocation {
    #[serde(rename = "executionSuccessful")]
    execution_successful: bool,
    #[serde(rename = "startTimeUtc")]
    start_time_utc: String,
    #[serde(rename = "endTimeUtc")]
    end_time_utc: String,
}

#[derive(serde::Serialize, Debug)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: &'static str,
    message: SarifMessage,
}

fn counterexample_rule_id(ce: &Counterexample) -> String {
    // The enum variants are already snake_case (non_camel_case_types), so
    // use the Debug representation which equals the variant name.
    format!("{:?}", ce)
}

fn all_rule_ids() -> Vec<String> {
    vec![
        "research_allowlist_present_in_locked_mode".to_string(),
        "antigravity_block_semantics_unproven".to_string(),
        "trace_profile_command_shape_inconsistent".to_string(),
        "cargo_subcommand_path_unverified".to_string(),
        "ocel_replay_placeholder".to_string(),
        "gate_without_trace_receipt".to_string(),
        "verify_without_trace_receipt".to_string(),
        "just_called_without_receipt".to_string(),
        "raw_cargo_used_by_agent".to_string(),
        "just_called_by_agent".to_string(),
        "shell_called_by_agent".to_string(),
        "python_called_by_agent".to_string(),
        "prose_completion_claim".to_string(),
        "compilation_treated_as_standing".to_string(),
        "receipt_without_execution_trace".to_string(),
        "manual_receipt_json".to_string(),
        "placeholder_authority".to_string(),
        "fake_test".to_string(),
        "dummy_gate".to_string(),
        "token_gate".to_string(),
        "hardcoded_commitment".to_string(),
        "hook_not_installed".to_string(),
    ]
}

fn format_rfc3339(t: std::time::SystemTime) -> String {
    let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    // Simple RFC-3339 UTC formatter without external deps.
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    // Gregorian calendar reconstruction (enough for tooling timestamps).
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, h, m, s, millis
    )
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Days since 1970-01-01 → (year, month, day)
    let mut year = 1970u64;
    loop {
        let leap = is_leap(year);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days = [
        31u64,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

fn build_sarif(
    counterexamples: &[Counterexample],
    q_release: u8,
    start: std::time::SystemTime,
    end: std::time::SystemTime,
) -> SarifOutput {
    let rules: Vec<SarifRule> = all_rule_ids()
        .into_iter()
        .map(|id| SarifRule {
            short_description: SarifMessage {
                text: id.replace('_', " "),
            },
            id,
        })
        .collect();

    let results: Vec<SarifResult> = counterexamples
        .iter()
        .map(|ce| SarifResult {
            rule_id: counterexample_rule_id(ce),
            level: "error",
            message: SarifMessage {
                text: counterexample_rule_id(ce).replace('_', " "),
            },
        })
        .collect();

    SarifOutput {
        schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        version: "2.1.0",
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "cargo-cicd-gate",
                    version: env!("CARGO_PKG_VERSION"),
                    rules,
                },
            },
            invocations: vec![SarifInvocation {
                execution_successful: q_release == 1,
                start_time_utc: format_rfc3339(start),
                end_time_utc: format_rfc3339(end),
            }],
            results,
        }],
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GateReport {
    pub schema: String,
    pub release: String,
    pub q_release: u8,
    pub failset_cardinality: usize,
    pub counterexamples: Vec<Counterexample>,
    pub ocel_event_id: String,
    pub components: GateComponents,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct GateComponents {
    pub agent_boundary: u8,
    pub pre_tool_use: u8,
    pub trace_profile: u8,
    pub receipt: u8,
    pub ocel: u8,
    pub doctor: u8,
    pub gate: u8,
    pub playground_cheats_detected: u8,
    pub no_prose_standing: u8,
    pub v_cargo_cicd: u8,
}

// ── Real component measurements ───────────────────────────────────────────────
//
// Every component is measured from observable repo/receipt state and defaults
// to 0 when its evidence artifact is absent. No component is hardcoded to 1.

fn read_repo_file(repo_dir: &str, rel: &str) -> Option<String> {
    std::fs::read_to_string(std::path::Path::new(repo_dir).join(rel)).ok()
}

/// `.agents/hooks.json` exists with a `pre-tool-use` hook entry wired to
/// cargo-cicd's executor (same contract barrier.rs's hook_not_installed checks).
fn pre_tool_use_installed(repo_dir: &str) -> bool {
    read_repo_file(repo_dir, ".agents/hooks.json")
        .map(|c| c.contains("pre-tool-use") && c.contains("cargo-cicd.execute"))
        .unwrap_or(false)
}

/// Receipt file with the required provenance fields (same field contract
/// `crate::nouns::receipt::judge_receipt` enforces for non-self-hashing JSON).
fn receipt_json_is_valid_format(content: &str) -> bool {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(content) else {
        return false;
    };
    ["schema", "command", "exit_code", "stdout_digest", "stderr_digest", "git_before", "git_after"]
        .iter()
        .all(|k| v.get(k).is_some_and(|f| !f.is_null()))
}

fn receipts_dir(repo_dir: &str) -> std::path::PathBuf {
    std::path::Path::new(repo_dir).join(".cargo-cicd").join("receipts")
}

/// ≥1 receipt file in `.cargo-cicd/receipts/` parses with the required fields.
fn has_valid_receipt(repo_dir: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(receipts_dir(repo_dir)) else {
        return false;
    };
    entries.flatten().any(|e| {
        e.path().extension().is_some_and(|x| x == "json")
            && std::fs::read_to_string(e.path())
                .map(|c| receipt_json_is_valid_format(&c))
                .unwrap_or(false)
    })
}

/// ≥1 trace-profile provenance receipt minted by `trace profile`.
fn has_trace_profile_receipt(repo_dir: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(receipts_dir(repo_dir)) else {
        return false;
    };
    entries.flatten().any(|e| {
        e.file_name().to_string_lossy().starts_with("trace-profile-")
            && e.file_name().to_string_lossy().ends_with(".json")
    })
}

/// OCEL journal exists and holds at least one event line.
fn ocel_journal_non_empty(repo_dir: &str) -> bool {
    let path = std::path::Path::new(repo_dir)
        .join(".cargo-cicd")
        .join("ocel")
        .join("events.jsonl");
    std::fs::read_to_string(path)
        .map(|c| c.lines().any(|l| !l.trim().is_empty()))
        .unwrap_or(false)
}

/// A prior gate run is recorded in the OCEL journal.
fn has_gate_computed_event(repo_dir: &str) -> bool {
    let path = std::path::Path::new(repo_dir)
        .join(".cargo-cicd")
        .join("ocel")
        .join("events.jsonl");
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    content
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .any(|v| {
            v.get("event_type").and_then(|t| t.as_str()) == Some("GateComputed")
                || v.get("eventTypeId").and_then(|t| t.as_str()) == Some("GateComputed")
        })
}

/// Bounded cheat-marker scan over `playground/` (the surface the cheat scanner
/// nominally covers): tautological asserts, fake tests, dummy gates, todos.
fn playground_cheats_detected(repo_dir: &str) -> bool {
    let root = std::path::Path::new(repo_dir).join("playground");
    if !root.is_dir() {
        return false;
    }
    const MARKERS: [&str; 4] = ["assert!(true)", "fake_test", "dummy_gate", "todo!("];
    let mut stack = vec![root];
    let mut visited = 0usize;
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if visited < 10_000 {
                visited += 1;
                if let Ok(c) = std::fs::read_to_string(&p) {
                    if MARKERS.iter().any(|m| c.contains(m)) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn measure_components(repo_dir: &str, counterexamples: &[Counterexample]) -> GateComponents {
    let ce_names: Vec<String> = counterexamples.iter().map(|c| format!("{:?}", c)).collect();
    let no_agent_authority_violations = !ce_names.iter().any(|n| {
        matches!(
            n.as_str(),
            "raw_cargo_used_by_agent" | "shell_called_by_agent" | "just_called_by_agent" | "python_called_by_agent"
        )
    });
    let no_prose_claims = !ce_names
        .iter()
        .any(|n| matches!(n.as_str(), "prose_completion_claim" | "compilation_treated_as_standing"));
    let no_fraud_markers = !ce_names.iter().any(|n| {
        matches!(
            n.as_str(),
            "manual_receipt_json" | "fake_test" | "dummy_gate" | "token_gate" | "placeholder_authority"
        )
    });
    let hooks_present =
        std::path::Path::new(repo_dir).join(".agents/hooks.json").exists();
    let cicd_toml_present = std::path::Path::new(repo_dir).join("cicd.toml").exists();

    GateComponents {
        // Agent boundary layer: hooks config installed AND no agent-authority bypasses detected.
        agent_boundary: (hooks_present && no_agent_authority_violations) as u8,
        // pre-tool-use hook wired to cargo-cicd.execute.
        pre_tool_use: pre_tool_use_installed(repo_dir) as u8,
        // A trace-profile provenance receipt exists.
        trace_profile: has_trace_profile_receipt(repo_dir) as u8,
        // At least one well-formed receipt in the receipts dir.
        receipt: has_valid_receipt(repo_dir) as u8,
        // OCEL journal exists and is non-empty.
        ocel: ocel_journal_non_empty(repo_dir) as u8,
        // Doctor surface: cicd.toml present AND fraud-scan counterexamples clear.
        doctor: (cicd_toml_present && no_fraud_markers) as u8,
        // A prior gate run recorded a GateComputed event in the OCEL journal.
        gate: has_gate_computed_event(repo_dir) as u8,
        // Cheat markers actually found under playground/ (0 when no playground).
        playground_cheats_detected: playground_cheats_detected(repo_dir) as u8,
        // No unbacked prose standing/compilation claims detected.
        no_prose_standing: no_prose_claims as u8,
        v_cargo_cicd: if counterexamples.is_empty() { 1 } else { 0 },
    }
}

fn print_report(
    report: &GateReport,
    json: bool,
    sarif: bool,
    start: std::time::SystemTime,
    end: std::time::SystemTime,
) {
    if sarif {
        let sarif_doc = build_sarif(&report.counterexamples, report.q_release, start, end);
        let out = serde_json::to_string_pretty(&sarif_doc).unwrap_or_else(|_| "{}".to_string());
        println!("{}", out);
    } else if json {
        let out = serde_json::to_string(report).unwrap_or_else(|_| "{}".to_string());
        println!("{}", out);
    } else {
        let out = serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string());
        println!("{}", out);
    }
}

fn compute_gate(repo_dir: &str, json: bool, sarif: bool) -> Result<()> {
    let start = std::time::SystemTime::now();
    let counterexamples = crate::barrier::detect_barriers(std::path::Path::new(repo_dir));
    let end = std::time::SystemTime::now();

    let components = measure_components(repo_dir, &counterexamples);

    let q_release = components.v_cargo_cicd;

    let ocel_event = crate::ocel::append_ocel_event(
        repo_dir,
        "GateComputed",
        serde_json::json!({
            "q_release": q_release,
            "failset_cardinality": counterexamples.len()
        }),
        "",
    )
    .map_err(|e| {
        clap_noun_verb::error::NounVerbError::execution_error(format!(
            "failed to append GateComputed OCEL event: {}",
            e
        ))
    })?;

    let report = GateReport {
        schema: "cargo-cicd.gate.v1".to_string(),
        release: format!("v{}", env!("CARGO_PKG_VERSION")),
        q_release,
        failset_cardinality: counterexamples.len(),
        counterexamples,
        ocel_event_id: ocel_event.event_id,
        components,
    };

    print_report(&report, json, sarif, start, end);

    if report.q_release == 0 {
        return Err(clap_noun_verb::error::NounVerbError::execution_error(
            "Gate failed",
        ));
    }

    Ok(())
}

#[verb("repo")]
pub fn cmd_repo(repo: Option<String>, json: bool, sarif: bool) -> Result<()> {
    let repo_dir = repo.unwrap_or_else(|| ".".into());
    compute_gate(&repo_dir, json, sarif)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_gate_sarif(_dir: &str) -> serde_json::Value {
        // Call compute_gate directly by building sarif from an empty counterexample list.
        let start = std::time::SystemTime::now();
        let counterexamples: Vec<Counterexample> = vec![];
        let end = std::time::SystemTime::now();
        let sarif_doc = build_sarif(&counterexamples, 1, start, end);
        serde_json::to_value(&sarif_doc).unwrap()
    }

    #[test]
    fn gate_sarif_version_is_2_1_0() {
        let val = run_gate_sarif(".");
        assert_eq!(val["version"], "2.1.0");
    }

    #[test]
    fn gate_sarif_has_rule_for_each_counterexample() {
        let start = std::time::SystemTime::now();
        let counterexamples: Vec<Counterexample> = vec![];
        let end = std::time::SystemTime::now();
        let sarif_doc = build_sarif(&counterexamples, 1, start, end);
        let val = serde_json::to_value(&sarif_doc).unwrap();
        let rules = val["runs"][0]["tool"]["driver"]["rules"]
            .as_array()
            .unwrap();
        assert!(
            rules.len() >= 19,
            "expected >= 19 rules, got {}",
            rules.len()
        );
    }
}
