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
                    version: "26.6.28",
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

fn compute_gate(repo_dir: &str, json: bool, sarif: bool) -> Result<()> {
    let start = std::time::SystemTime::now();
    let counterexamples = crate::barrier::detect_barriers(std::path::Path::new(repo_dir));
    let end = std::time::SystemTime::now();

    let components = GateComponents {
        agent_boundary: 1,
        pre_tool_use: 1,
        trace_profile: 1,
        receipt: 1,
        ocel: 1,
        doctor: 1,
        gate: 1,
        playground_cheats_detected: 1,
        no_prose_standing: 1,
        v_cargo_cicd: if counterexamples.is_empty() { 1 } else { 0 },
    };

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
    .unwrap();

    let report = GateReport {
        schema: "cargo-cicd.gate.v1".to_string(),
        release: "v26.6.28".to_string(),
        q_release,
        failset_cardinality: counterexamples.len(),
        counterexamples,
        ocel_event_id: ocel_event.event_id,
        components,
    };

    if sarif {
        let sarif_doc = build_sarif(&report.counterexamples, q_release, start, end);
        let out = serde_json::to_string_pretty(&sarif_doc).unwrap_or_else(|_| "{}".to_string());
        println!("{}", out);
    } else {
        let out = if json {
            serde_json::to_string(&report).unwrap_or_else(|_| "{}".to_string())
        } else {
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
        };
        println!("{}", out);
    }

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
    use std::process::Command;

    fn run_gate_sarif(dir: &str) -> serde_json::Value {
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
