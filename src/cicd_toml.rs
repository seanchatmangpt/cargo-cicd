use anyhow::Result;
use serde::{Deserialize, Serialize};
use star_toml::loader::{ConfigLifecycle, TrustedLoader};
use star_toml::{AdmittedConfig, Validate, Validator};
use std::path::Path;

/// v26.6.2 cicd.toml schema — the carrier contract
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct CicdToml {
    pub workspace: WorkspaceSection,
    pub state: StateSection,
    pub target: TargetSection,
    #[serde(rename = "test")]
    pub test: TestSection,
    #[serde(rename = "trybuild")]
    pub trybuild: TrybuildSection,
    #[serde(rename = "git")]
    pub git: GitSection,
    pub autonomic: AutonomicSection,
    #[serde(default)]
    pub events: Vec<EventRecord>,
    /// The standing compiler's ingestion/gate configuration. Optional: a
    /// `cicd.toml` written before v26.7.4 has no `[standing]` section at
    /// all, and must still parse.
    #[serde(default)]
    pub standing: StandingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceSection {
    pub name: String,
    pub toolchain: String,
    pub target_dir: String,
}

impl Default for WorkspaceSection {
    fn default() -> Self {
        Self {
            name: detect_workspace_name(),
            toolchain: detect_toolchain(),
            target_dir: "target".into(),
        }
    }
}

fn detect_workspace_name() -> String {
    std::fs::read_to_string("Cargo.toml")
        .ok()
        .and_then(|s| {
            toml::from_str::<toml::Value>(&s).ok().and_then(|v| {
                v.get("package")
                    .or_else(|| v.get("workspace").and_then(|w| w.get("package")))
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string())
            })
        })
        .unwrap_or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "workspace".into())
        })
}

fn detect_toolchain() -> String {
    // Check rust-toolchain.toml or rust-toolchain
    if let Ok(content) = std::fs::read_to_string("rust-toolchain.toml") {
        if let Some(line) = content.lines().find(|l| l.contains("channel")) {
            if let Some(ch) = line.split('"').nth(1) {
                return ch.to_string();
            }
        }
    }
    if let Ok(content) = std::fs::read_to_string("rust-toolchain") {
        return content.trim().to_string();
    }
    "stable".into()
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct StateSection {
    pub dirty: bool,
    pub target_size_gb: f64,
    pub changed_files: usize,
    pub changed_tests: usize,
    pub changed_trybuild_fixtures: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TargetSection {
    pub max_size_gb: u32,
    pub prune_after_days: u32,
}

impl Default for TargetSection {
    fn default() -> Self {
        Self {
            max_size_gb: 20,
            prune_after_days: 14,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct TestSection {
    pub changed: TestChangedSection,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TestChangedSection {
    pub enabled: bool,
    pub base: String,
}

impl Default for TestChangedSection {
    fn default() -> Self {
        Self {
            enabled: true,
            base: "origin/main".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct TrybuildSection {
    pub changed: TrybuildChangedSection,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TrybuildChangedSection {
    pub enabled: bool,
    pub snapshot_mode: String,
}

impl Default for TrybuildChangedSection {
    fn default() -> Self {
        Self {
            enabled: true,
            snapshot_mode: "changed-only".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct GitSection {
    pub phase: GitPhaseSection,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct GitPhaseSection {
    pub require_clean_tree: bool,
    pub commit_after_phase: bool,
}

impl Default for GitPhaseSection {
    fn default() -> Self {
        Self {
            require_clean_tree: true,
            commit_after_phase: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AutonomicSection {
    pub enabled: bool,
    pub mode: String,
}

impl Default for AutonomicSection {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: "suggest".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventRecord {
    pub kind: String,
    pub verdict: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl EventRecord {
    pub fn status_pass() -> Self {
        Self {
            kind: "status".into(),
            verdict: "pass".into(),
            details: None,
            timestamp: None,
        }
    }
}

// ── [standing] section ──────────────────────────────────────────────────

/// Configuration for the standing compiler (`cargo cicd standing ...`).
///
/// Every field is optional so an existing `cicd.toml` without a `[standing]`
/// section keeps parsing, and a partial section (e.g. just
/// `doctor_command`) is valid too — missing ingestors simply produce their
/// tolerant `UNSEEN`/`DISCOVERED` fallback artifacts at refresh time.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct StandingConfig {
    /// Shell command that prints `doctor check --format json` shaped
    /// output: `{build, config, frontier, receipts, tools, features}`.
    pub doctor_command: Option<String>,
    /// Glob patterns for OCEL process-validation JSON logs
    /// (`{is_conforming, fitness, violations, ...}`).
    pub ocel_logs: Vec<String>,
    /// Glob patterns for OCEL process-validation JSON — kept distinct from
    /// `ocel_logs` so raw OCEL event logs and conformance-check outputs can
    /// be pointed at different directories.
    pub process_validation: Vec<String>,
    /// Paths to JSONL receipt ledgers (one record per line, each carrying
    /// `chain_hash_hex`).
    pub receipt_ledgers: Vec<String>,
    /// Glob for `plan.json` plan-run artifacts carrying `powl_chain_hash`.
    pub plan_runs_glob: Option<String>,
    /// Glob for raw benchmark output files (existence/size/hash only — no
    /// divan table parsing).
    pub bench_raw_glob: Option<String>,
    /// Paths to markdown claim-promotion tables. Ingested as informational
    /// evidence only, never authoritative standing.
    pub claim_tables: Vec<String>,
    /// Client build targets: each gets its `build_command` shelled out
    /// with a timeout during `standing refresh`.
    pub clients: Vec<cargo_cicd_core::standing::sources::ClientTarget>,
    /// Directory of release docs consulted by the `claude-context` verb.
    pub release_docs_dir: Option<String>,
    /// Per-release required artifact+status pairs, checked by the
    /// `gate release` verb.
    pub release_gates: std::collections::BTreeMap<String, Vec<RequiredArtifactStatus>>,
}

/// One required `(artifact_id, status)` pair for a release gate.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RequiredArtifactStatus {
    pub artifact_id: String,
    pub status: cargo_cicd_core::standing::StandingStatus,
}

impl CicdToml {
    /// Load configuration from a TOML file.
    pub fn from_file(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Write configuration to a TOML file (compat alias).
    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        self.write(path)
    }

    /// Build from current workspace state
    pub fn from_current_workspace() -> Self {
        let mut cicd = Self::default();
        cicd.events.push(EventRecord::status_pass());
        cicd
    }

    /// Write to the given path
    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize cicd.toml: {}", e))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Write to ./cicd.toml
    pub fn write_default(&self) -> Result<()> {
        self.write("cicd.toml")
    }
}

impl Validate for CicdToml {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("workspace.name", &self.workspace.name);
        let tc = &self.workspace.toolchain;
        let valid_toolchain = matches!(tc.as_str(), "stable" | "beta" | "nightly")
            || tc.starts_with("nightly-") // dated rustup channel, e.g. nightly-2026-06-22
            || tc
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false);
        v.check_predicate(
            "workspace.toolchain",
            valid_toolchain,
            "invalid_toolchain",
            "must be stable, beta, nightly, a dated nightly (nightly-YYYY-MM-DD), or a version string starting with a digit",
        );
        v.check_non_empty("test.changed.base", &self.test.changed.base);
        v.check_predicate(
            "target.max_size_gb",
            self.target.max_size_gb > 0,
            "invalid_max_size_gb",
            "must be greater than 0",
        );
    }
}

impl ConfigLifecycle for CicdToml {}

/// Load and admit the cicd.toml configuration through the star-toml pipeline.
/// Returns a typestate-sealed AdmittedConfig that carries a BLAKE3 witness hash.
pub fn load_admitted() -> Result<AdmittedConfig<CicdToml>> {
    let loader = TrustedLoader::new()
        .layer_file_if_exists("cicd.toml")
        .env_prefix("CICD_");
    let admitted = loader.load_admitted::<CicdToml>()?;
    Ok(admitted)
}

/// Load cicd.toml without admission — fallback for contexts where the
/// typestate guarantee is not yet required.
pub fn load_or_default() -> CicdToml {
    CicdToml::from_file(std::path::Path::new("cicd.toml")).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_cicd_toml_admits() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cicd.toml");
        let config = CicdToml::default();
        config.write(&path).expect("write failed");
        // Use from_file + check() to prove the validation pipeline.
        let loaded = CicdToml::from_file(&path).expect("read failed");
        loaded.check().expect("valid config should pass validation");
    }

    #[test]
    fn empty_name_fails_validate() {
        let mut config = CicdToml::default();
        config.workspace.name = String::new();
        assert!(
            config.check().is_err(),
            "expected validation errors for empty name"
        );
    }

    #[test]
    fn unknown_field_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cicd.toml");
        let contents = r#"
[workspace]
name = "test"
toolchain = "stable"
target_dir = "target"
frob = true

[state]
dirty = false
target_size_gb = 0.0
changed_files = 0
changed_tests = 0
changed_trybuild_fixtures = 0

[target]
max_size_gb = 20
prune_after_days = 14

[test.changed]
enabled = true
base = "origin/main"

[trybuild.changed]
enabled = true
snapshot_mode = "changed-only"

[git.phase]
require_clean_tree = true
commit_after_phase = false

[autonomic]
enabled = true
mode = "suggest"
"#;
        std::fs::write(&path, contents).unwrap();
        let result = CicdToml::from_file(&path);
        assert!(result.is_err(), "expected Err for unknown field");
    }
}
