//! Schema-based analyzer for cicd.toml.
//!
//! Validates cicd.toml against the bundled JSON schema and emits
//! CICD-SCHEMA-001 (unknown field) or CICD-SCHEMA-002 (constraint violation).

use cargo_cicd_core::diagnostics::{CicdCode, CicdFinding, CicdSeverity};
use cargo_cicd_core::workspace::WorkspaceSnapshot;

use super::CicdAnalyzer;

/// Analyzer that validates cicd.toml against the bundled JSON schema.
pub struct CicdTomlSchemaAnalyzer;

impl CicdAnalyzer for CicdTomlSchemaAnalyzer {
    fn name(&self) -> &'static str {
        "cicd-toml-schema"
    }

    fn analyze(&self, snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
        analyze_schema(snapshot)
    }
}

fn analyze_schema(snapshot: &WorkspaceSnapshot) -> Vec<CicdFinding> {
    // Load the bundled schema (include_str! at compile time).
    let schema_str = include_str!("../../../../schemas/cicd-toml-v1.json");
    if schema_str.is_empty() {
        tracing::warn!("cicd-toml-v1.json schema is empty; skipping schema validation");
        return Vec::new();
    }

    let schema_value: serde_json::Value = match serde_json::from_str(schema_str) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Failed to parse bundled cicd.toml schema: {}", e);
            return Vec::new();
        }
    };

    // Locate cicd.toml in the workspace root.
    let cicd_toml_path = snapshot.root.join("cicd.toml");
    let toml_source = match std::fs::read_to_string(&cicd_toml_path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    // Parse as toml::Value then convert to serde_json::Value.
    let toml_value: toml::Value = match toml_source.parse() {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Failed to parse cicd.toml as TOML: {}", e);
            return Vec::new();
        }
    };

    let instance = match serde_json::to_value(&toml_value) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                "Failed to convert cicd.toml to JSON for schema validation: {}",
                e
            );
            return Vec::new();
        }
    };

    // Run the validator.
    let validator = match jsonschema::validator_for(&schema_value) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Failed to compile cicd.toml schema: {}", e);
            return Vec::new();
        }
    };

    let source_lines: Vec<&str> = toml_source.lines().collect();

    let mut findings = Vec::new();
    for error in validator.iter_errors(&instance) {
        let path_str = error.instance_path.to_string();
        let field_name = path_str.split('/').last().unwrap_or("<root>").to_string();

        // Find the line number by scanning source for the field name.
        let line_no: u32 = source_lines
            .iter()
            .enumerate()
            .find(|(_, line)| {
                line.trim_start().starts_with(&format!("{}  =", field_name))
                    || line.trim_start().starts_with(&format!("{} =", field_name))
                    || line.trim_start().starts_with(&format!("{}=", field_name))
                    || line
                        .trim_start()
                        .starts_with(&format!("\"{}\"", field_name))
            })
            .map(|(i, _)| i as u32)
            .unwrap_or(0);

        // Classify the error as unknown field or constraint violation.
        let error_str = error.to_string();
        let (code, severity) = if error_str.contains("Additional properties")
            || error_str.contains("additionalProperties")
        {
            (CicdCode::CicdTomlUnknownField, CicdSeverity::Warning)
        } else {
            (CicdCode::CicdTomlConstraintViolation, CicdSeverity::Error)
        };

        let finding = CicdFinding::new(
            code,
            cicd_toml_path.to_string_lossy().as_ref(),
            "cicd-toml-schema",
            vec![code.repair_hint().to_string()],
            format!("cicd.toml schema error at `{}`: {}", field_name, error_str),
        )
        .with_severity(severity)
        .at_line(line_no);

        findings.push(finding);
    }

    findings
}
