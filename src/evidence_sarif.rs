//! SARIF v2.1.0 output for cargo-cicd evidence diagnostics.
//!
//! Implements [`SarifReport`] (serializable to valid SARIF 2.1.0 JSON) and
//! [`evidence_issues_to_sarif`] which converts a slice of [`EvidenceIssue`]
//! into a report suitable for writing to `results.sarif.json`.

use serde::{Deserialize, Serialize};

// ── EvidenceIssue ─────────────────────────────────────────────────────────────

/// Severity of an evidence diagnostic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,
    Warning,
    Note,
}

impl IssueSeverity {
    /// Map to the SARIF `level` string.
    pub fn sarif_level(&self) -> &'static str {
        match self {
            IssueSeverity::Error => "error",
            IssueSeverity::Warning => "warning",
            IssueSeverity::Note => "note",
        }
    }

    /// Map to a SARIF rule id.
    pub fn rule_id(&self) -> &'static str {
        match self {
            IssueSeverity::Error => "CICD-E1",
            IssueSeverity::Warning => "CICD-W1",
            IssueSeverity::Note => "CICD-N1",
        }
    }
}

/// A single evidence diagnostic produced by the audit pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceIssue {
    /// Human-readable description of the issue.
    pub message: String,
    /// Diagnostic severity.
    pub severity: IssueSeverity,
    /// ISO-8601 UTC timestamp when this issue was first detected.
    pub detected_at: String,
    /// Optional file path associated with the issue.
    pub file_path: Option<String>,
}

impl EvidenceIssue {
    /// Construct a new error-level evidence issue.
    pub fn error(message: impl Into<String>, detected_at: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: IssueSeverity::Error,
            detected_at: detected_at.into(),
            file_path: None,
        }
    }

    /// Construct a new warning-level evidence issue.
    pub fn warning(message: impl Into<String>, detected_at: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: IssueSeverity::Warning,
            detected_at: detected_at.into(),
            file_path: None,
        }
    }
}

// ── SARIF 2.1.0 types ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct SarifMessage {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SarifArtifactLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    pub artifact_location: SarifArtifactLocation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    pub physical_location: SarifPhysicalLocation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SarifResultProperties {
    #[serde(rename = "firstDetectionTimeUtc")]
    pub first_detection_time_utc: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SarifResult {
    #[serde(rename = "ruleId")]
    pub rule_id: String,
    pub level: String,
    pub message: SarifMessage,
    pub locations: Vec<SarifLocation>,
    pub properties: SarifResultProperties,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SarifRule {
    pub id: String,
    pub name: String,
    #[serde(rename = "shortDescription")]
    pub short_description: SarifMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SarifDriver {
    pub name: String,
    pub version: String,
    pub rules: Vec<SarifRule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SarifTool {
    pub driver: SarifDriver,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SarifInvocation {
    #[serde(rename = "executionSuccessful")]
    pub execution_successful: bool,
    #[serde(rename = "startTimeUtc")]
    pub start_time_utc: String,
    #[serde(rename = "endTimeUtc")]
    pub end_time_utc: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SarifRun {
    pub tool: SarifTool,
    pub invocations: Vec<SarifInvocation>,
    pub results: Vec<SarifResult>,
}

/// A SARIF 2.1.0 report, serializable to JSON.
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifReport {
    pub version: String,
    #[serde(rename = "$schema")]
    pub schema: String,
    pub runs: Vec<SarifRun>,
}

// ── Conversion ────────────────────────────────────────────────────────────────

/// Convert a slice of [`EvidenceIssue`] into a [`SarifReport`].
///
/// - `issues` — the diagnostics to include as SARIF results.
/// - `start_time` — ISO-8601 UTC start time of the audit run.
/// - `end_time` — ISO-8601 UTC end time of the audit run.
/// - `run_ordinal` — monotonic counter distinguishing runs in a session
///   (used to determine `executionSuccessful`; a run with zero error-level
///   issues is considered successful).
pub fn evidence_issues_to_sarif(
    issues: &[EvidenceIssue],
    start_time: &str,
    end_time: &str,
    _run_ordinal: usize,
) -> SarifReport {
    let execution_successful = issues.iter().all(|i| i.severity != IssueSeverity::Error);

    // Collect the distinct rule ids that appear in this issue set.
    let mut rule_ids: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for issue in issues {
        rule_ids.insert(issue.severity.rule_id());
    }

    let rules: Vec<SarifRule> = rule_ids
        .iter()
        .map(|&id| SarifRule {
            id: id.to_string(),
            name: id.to_string(),
            short_description: SarifMessage {
                text: format!("cargo-cicd evidence diagnostic: {}", id),
            },
        })
        .collect();

    let results: Vec<SarifResult> = issues
        .iter()
        .map(|issue| {
            let location = SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: issue.file_path.clone(),
                    },
                },
            };
            SarifResult {
                rule_id: issue.severity.rule_id().to_string(),
                level: issue.severity.sarif_level().to_string(),
                message: SarifMessage {
                    text: issue.message.clone(),
                },
                locations: vec![location],
                properties: SarifResultProperties {
                    first_detection_time_utc: issue.detected_at.clone(),
                },
            }
        })
        .collect();

    SarifReport {
        version: "2.1.0".to_string(),
        schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json".to_string(),
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "cargo-cicd".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    rules,
                },
            },
            invocations: vec![SarifInvocation {
                execution_successful,
                start_time_utc: start_time.to_string(),
                end_time_utc: end_time.to_string(),
            }],
            results,
        }],
    }
}

/// Write a [`SarifReport`] to `target/cargo-cicd/evidence/results.sarif.json`.
pub fn write_sarif_report(report: &SarifReport) -> anyhow::Result<std::path::PathBuf> {
    let dir = std::path::PathBuf::from("target/cargo-cicd/evidence");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("results.sarif.json");
    std::fs::write(&path, serde_json::to_string_pretty(report)?)?;
    Ok(path)
}

// ── Tests ─────────────────────────────────────────────────────────────────────
