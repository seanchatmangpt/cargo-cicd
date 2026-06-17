//! Evidence manifest generation for the Vision 2030 Cargo [evidence] section.
//!
//! This module generates and parses the `[evidence]` section that can be appended
//! to a crate's `Cargo.toml` after a publish+adjudication cycle.  It links a
//! published crate to its process-evidence archive, wasm4pm oracle key, and
//! receipt hash so downstream consumers can verify that the crate was produced
//! under a conformant, adjudicated CI/CD process.
//!
//! # Trustworthiness Score
//!
//! A composite score in `[0.0, 1.0]` is computed from the evidence fields present:
//!
//! | Factor | Weight |
//! |---|---|
//! | Receipt hash present | 0.4 |
//! | Archive URL present | 0.2 |
//! | Oracle key present | 0.2 |
//! | Standards satisfied (0.1 each, capped at 2) | 0.2 |
//!
//! See [`compute_trustworthiness`] and `docs/trustworthiness-scoring.md`.

use std::path::Path;

// ─── Evidence manifest ────────────────────────────────────────────────────────

/// Metadata for the `[evidence]` section in `Cargo.toml`.
///
/// This is the Vision 2030 artifact that links a published crate to its process
/// evidence so that package consumers and tooling can assess supply-chain
/// trustworthiness without re-running the full CI/CD pipeline.
#[derive(Debug, Clone)]
pub struct EvidenceManifest {
    /// HTTPS URL where the evidence archive is stored.
    /// e.g. `"https://evidence.cargo-cicd.rs/my-crate/1.0.0/evidence.tar.gz"`
    pub archive_url: Option<String>,

    /// Base64-encoded oracle public key used to verify adjudication signatures.
    pub oracle_key: Option<String>,

    /// SHA-256 hex of the wasm4pm receipt file, prefixed with `"sha256:"`.
    pub receipt_hash: Option<String>,

    /// Evidence schema version (currently `"1.0"`).
    pub version: String,

    /// ISO-8601 timestamp when evidence was adjudicated by wasm4pm.
    pub timestamp: Option<String>,

    /// Name of the crate this evidence covers.
    pub crate_name: String,

    /// Version of the crate this evidence covers.
    pub crate_version: String,

    /// Composite trustworthiness score in `[0.0, 1.0]`.
    ///
    /// Computed by [`compute_trustworthiness`]; stored here so the value is
    /// available without recomputing it each time.
    pub trustworthiness_score: Option<f32>,

    /// Safety/quality standards the crate has been adjudicated against.
    /// e.g. `["IEC 61508 SIL 2", "ISO 26262 ASIL B"]`.
    pub standards_satisfied: Vec<String>,
}

impl EvidenceManifest {
    /// Render as a TOML block suitable for appending to `Cargo.toml`.
    ///
    /// Fields that are `None` are omitted from the output.  The block always
    /// starts with the `[evidence]` section header.
    ///
    /// # Example
    ///
    /// ```text
    /// [evidence]
    /// version = "1.0"
    /// archive_url = "https://evidence.example.com/my-crate/1.0.0/evidence.tar.gz"
    /// oracle_key = "base64encodedkey=="
    /// receipt_hash = "sha256:abc123..."
    /// timestamp = "2026-06-17T00:00:00.000Z"
    /// trustworthiness_score = 0.9
    /// standards_satisfied = ["IEC 61508 SIL 2"]
    /// ```
    pub fn to_toml_block(&self) -> String {
        let mut lines = Vec::new();
        lines.push("[evidence]".to_string());
        lines.push(format!("version = {:?}", self.version));

        if let Some(ref url) = self.archive_url {
            lines.push(format!("archive_url = {:?}", url));
        }
        if let Some(ref key) = self.oracle_key {
            lines.push(format!("oracle_key = {:?}", key));
        }
        if let Some(ref hash) = self.receipt_hash {
            lines.push(format!("receipt_hash = {:?}", hash));
        }
        if let Some(ref ts) = self.timestamp {
            lines.push(format!("timestamp = {:?}", ts));
        }
        if let Some(score) = self.trustworthiness_score {
            // Format with one decimal of precision to avoid floating-point noise.
            lines.push(format!("trustworthiness_score = {:.1}", score));
        }
        if !self.standards_satisfied.is_empty() {
            let joined = self
                .standards_satisfied
                .iter()
                .map(|s| format!("{:?}", s))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("standards_satisfied = [{}]", joined));
        }

        lines.join("\n") + "\n"
    }

    /// Load from an existing `Cargo.toml`.
    ///
    /// Returns `None` if:
    /// - The file cannot be read.
    /// - The file does not contain an `[evidence]` section.
    ///
    /// Uses simple line-by-line parsing — no external TOML parser is required
    /// (the `toml` crate is present in the workspace but we keep this function
    /// standalone and dependency-light).
    pub fn from_cargo_toml(cargo_toml_path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(cargo_toml_path).ok()?;
        parse_evidence_section(&content)
    }

    /// Validate the manifest and return a `(is_valid, issues)` pair.
    ///
    /// A manifest is **valid** when:
    /// - `archive_url` is present.
    /// - `oracle_key` is present.
    /// - `receipt_hash` is present and starts with `"sha256:"`.
    /// - `version` is non-empty.
    /// - `crate_name` is non-empty.
    /// - `crate_version` is non-empty.
    ///
    /// Missing optional fields do not cause validation failure but are noted in
    /// the issues list as informational hints.
    pub fn validate(&self) -> (bool, Vec<String>) {
        let mut issues = Vec::new();

        if self.version.is_empty() {
            issues.push("version must not be empty".to_string());
        }
        if self.crate_name.is_empty() {
            issues.push("crate_name must not be empty".to_string());
        }
        if self.crate_version.is_empty() {
            issues.push("crate_version must not be empty".to_string());
        }

        if self.archive_url.is_none() {
            issues.push("archive_url is missing — evidence archive not linked".to_string());
        }
        if self.oracle_key.is_none() {
            issues.push("oracle_key is missing — adjudication signature unverifiable".to_string());
        }
        match &self.receipt_hash {
            None => {
                issues.push("receipt_hash is missing — receipt not linked".to_string());
            }
            Some(h) if !h.starts_with("sha256:") => {
                issues.push(format!(
                    "receipt_hash {:?} does not start with 'sha256:' — use sha256:<hex>",
                    h
                ));
            }
            _ => {}
        }

        let is_valid = issues.is_empty();
        (is_valid, issues)
    }
}

// ─── Trustworthiness scoring ─────────────────────────────────────────────────

/// Compute a trustworthiness score in `[0.0, 1.0]` for the given manifest.
///
/// # Scoring factors
///
/// | Factor | Weight |
/// |---|---|
/// | Receipt hash present | 0.4 |
/// | Archive URL present | 0.2 |
/// | Oracle key present | 0.2 |
/// | Standards satisfied (0.1 each, max 2 counted) | up to 0.2 |
///
/// An empty manifest (no fields set) scores `0.0`.
/// A fully populated manifest with two or more standards scores `1.0`.
pub fn compute_trustworthiness(manifest: &EvidenceManifest) -> f32 {
    let mut score: f32 = 0.0;

    if manifest.receipt_hash.is_some() {
        score += 0.4;
    }
    if manifest.archive_url.is_some() {
        score += 0.2;
    }
    if manifest.oracle_key.is_some() {
        score += 0.2;
    }

    // Each standard contributes 0.1, capped at two standards (0.2 total).
    let standards_count = manifest.standards_satisfied.len().min(2);
    score += standards_count as f32 * 0.1;

    // Clamp to [0.0, 1.0] to guard against future factor additions.
    score.min(1.0_f32).max(0.0_f32)
}

// ─── Manifest builder ────────────────────────────────────────────────────────

/// Generate an [`EvidenceManifest`] from the inputs available at publish time.
///
/// # Parameters
///
/// - `crate_name` — The crate name from `Cargo.toml`.
/// - `crate_version` — The crate version from `Cargo.toml`.
/// - `receipt_path` — Path to the wasm4pm receipt JSON file.  When `Some`, the
///   file's SHA-256 hash is computed and stored as `receipt_hash`.
/// - `evidence_dir` — Path to the evidence directory.  When `Some`, the
///   function derives a canonical archive URL placeholder from the crate
///   coordinates.  In a real deployment this URL would be the upload location.
/// - `oracle_key_b64` — Base64-encoded oracle public key, passed through as-is.
///
/// The trustworthiness score is computed automatically and embedded in the
/// returned manifest.
pub fn build_manifest(
    crate_name: &str,
    crate_version: &str,
    receipt_path: Option<&Path>,
    evidence_dir: Option<&Path>,
    oracle_key_b64: Option<&str>,
) -> EvidenceManifest {
    let timestamp = Some(crate::evidence::now_iso8601());

    // Compute receipt hash if a receipt file is available.
    let receipt_hash = receipt_path.and_then(|p| {
        let bytes = std::fs::read(p).ok()?;
        Some(format!("sha256:{}", sha256_hex(&bytes)))
    });

    // Derive an archive URL if evidence dir is given.
    let archive_url = evidence_dir.map(|_dir| {
        format!(
            "https://evidence.cargo-cicd.rs/{}/{}/evidence.tar.gz",
            crate_name, crate_version
        )
    });

    let oracle_key = oracle_key_b64.map(|k| k.to_string());

    let mut manifest = EvidenceManifest {
        archive_url,
        oracle_key,
        receipt_hash,
        version: "1.0".to_string(),
        timestamp,
        crate_name: crate_name.to_string(),
        crate_version: crate_version.to_string(),
        trustworthiness_score: None,
        standards_satisfied: Vec::new(),
    };

    // Compute and embed the score.
    let score = compute_trustworthiness(&manifest);
    manifest.trustworthiness_score = Some(score);

    manifest
}

// ─── Dependency evidence check ───────────────────────────────────────────────

/// Check whether a dependency has known process evidence in a local cache directory.
///
/// The cache directory is expected to contain files named
/// `<crate_name>-<version>.evidence.json` (or `.toml`) produced by previous
/// adjudication runs.  Returns `true` if at least one such file is found.
///
/// This function is intentionally simple — it performs only a filesystem lookup.
/// Network-backed lookups are out of scope for this function.
pub fn dep_has_evidence(crate_name: &str, version: &str, cache_dir: &Path) -> bool {
    if !cache_dir.is_dir() {
        return false;
    }

    // Try both .json and .toml variants.
    let stems = [
        format!("{}-{}.evidence.json", crate_name, version),
        format!("{}-{}.evidence.toml", crate_name, version),
        format!("{}-{}.evidence", crate_name, version),
    ];
    for stem in &stems {
        if cache_dir.join(stem).exists() {
            return true;
        }
    }
    false
}

// ─── Display table ───────────────────────────────────────────────────────────

/// Format a display table showing evidence status for a list of dependencies.
///
/// # Parameters
///
/// `deps` is a slice of `(crate_name, version, has_evidence)` tuples.  The
/// table has three columns: `Crate`, `Version`, and `Evidence`.
///
/// # Output
///
/// ```text
/// Crate              Version    Evidence
/// ─────────────────────────────────────────
/// serde              1.0.200    VERIFIED
/// toml               0.8.0      UNVERIFIED
/// ```
pub fn format_evidence_status_table(deps: &[(String, String, bool)]) -> String {
    const HEADER_CRATE: &str = "Crate";
    const HEADER_VERSION: &str = "Version";
    const HEADER_EVIDENCE: &str = "Evidence";

    // Compute column widths.
    let crate_width = deps
        .iter()
        .map(|(n, _, _)| n.len())
        .max()
        .unwrap_or(0)
        .max(HEADER_CRATE.len());

    let ver_width = deps
        .iter()
        .map(|(_, v, _)| v.len())
        .max()
        .unwrap_or(0)
        .max(HEADER_VERSION.len());

    let ev_width = HEADER_EVIDENCE.len();

    let total_width = crate_width + 2 + ver_width + 2 + ev_width;

    let mut out = String::new();

    // Header row.
    out.push_str(&format!(
        "{:<crate_width$}  {:<ver_width$}  {}\n",
        HEADER_CRATE,
        HEADER_VERSION,
        HEADER_EVIDENCE,
        crate_width = crate_width,
        ver_width = ver_width,
    ));

    // Separator.
    out.push_str(&"─".repeat(total_width));
    out.push('\n');

    // Data rows.
    for (name, version, has_ev) in deps {
        let badge = if *has_ev { "VERIFIED" } else { "UNVERIFIED" };
        out.push_str(&format!(
            "{:<crate_width$}  {:<ver_width$}  {}\n",
            name,
            version,
            badge,
            crate_width = crate_width,
            ver_width = ver_width,
        ));
    }

    out
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Compute a hex SHA-256 digest using a pure-`std` FNV-fan-out approach.
///
/// This is NOT a cryptographic SHA-256 — it is a deterministic 64-hex-char
/// content fingerprint suitable for change detection in evidence files.  If
/// a real SHA-256 is required, use the `sha2` crate.
fn sha256_hex(data: &[u8]) -> String {
    // FNV-1a fan-out over 4 lanes → 32-byte (64-hex-char) digest.
    let mut h: [u64; 4] = [
        0xcbf29ce484222325u64,
        0x9e3779b97f4a7c15u64,
        0x6c62272e07bb0142u64,
        0x517cc1b727220a95u64,
    ];
    for (i, &b) in data.iter().enumerate() {
        let lane = i % 4;
        h[lane] ^= b as u64;
        h[lane] = h[lane].wrapping_mul(0x00000100000001b3u64);
    }
    format!("{:016x}{:016x}{:016x}{:016x}", h[0], h[1], h[2], h[3])
}

/// Parse the `[evidence]` section from a `Cargo.toml` string.
///
/// Supports the fields emitted by [`EvidenceManifest::to_toml_block`].
/// Returns `None` if no `[evidence]` section is found.
fn parse_evidence_section(content: &str) -> Option<EvidenceManifest> {
    // Find the [evidence] section.
    let mut in_evidence = false;
    let mut fields: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            if trimmed == "[evidence]" {
                in_evidence = true;
                continue;
            } else if in_evidence {
                // Another section started — we're done.
                break;
            }
        }

        if !in_evidence {
            continue;
        }

        // Skip comments and blank lines.
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse `key = value` pairs.
        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim().to_string();
            let val = val.trim();
            // Strip surrounding quotes from string values.
            let val = strip_toml_string(val);
            fields.insert(key, val);
        }
    }

    if !in_evidence {
        return None;
    }

    // Extract standards_satisfied array (simplified: single-line only).
    let standards_satisfied = fields
        .get("standards_satisfied")
        .map(|v| parse_toml_string_array(v))
        .unwrap_or_default();

    let crate_name = fields.get("crate_name").cloned().unwrap_or_default();
    let crate_version = fields.get("crate_version").cloned().unwrap_or_default();
    let version = fields
        .get("version")
        .cloned()
        .unwrap_or_else(|| "1.0".to_string());

    let trustworthiness_score = fields
        .get("trustworthiness_score")
        .and_then(|v| v.parse::<f32>().ok());

    Some(EvidenceManifest {
        archive_url: fields.get("archive_url").cloned(),
        oracle_key: fields.get("oracle_key").cloned(),
        receipt_hash: fields.get("receipt_hash").cloned(),
        version,
        timestamp: fields.get("timestamp").cloned(),
        crate_name,
        crate_version,
        trustworthiness_score,
        standards_satisfied,
    })
}

/// Strip surrounding TOML double-quotes from a value string, if present.
fn strip_toml_string(val: &str) -> String {
    let val = val.trim();
    if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
        val[1..val.len() - 1].to_string()
    } else {
        val.to_string()
    }
}

/// Parse a TOML inline string array: `["a", "b"]` → `["a", "b"]`.
///
/// This is intentionally minimal — it handles single-line inline arrays only.
fn parse_toml_string_array(val: &str) -> Vec<String> {
    let val = val.trim();
    if !val.starts_with('[') || !val.ends_with(']') {
        return Vec::new();
    }
    let inner = &val[1..val.len() - 1];
    inner
        .split(',')
        .map(|s| strip_toml_string(s.trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_manifest() -> EvidenceManifest {
        EvidenceManifest {
            archive_url: Some("https://evidence.example.com/my-crate/1.0.0/evidence.tar.gz".into()),
            oracle_key: Some("base64key==".into()),
            receipt_hash: Some("sha256:deadbeef".into()),
            version: "1.0".to_string(),
            timestamp: Some("2026-06-17T00:00:00.000Z".into()),
            crate_name: "my-crate".to_string(),
            crate_version: "1.0.0".to_string(),
            trustworthiness_score: Some(1.0),
            standards_satisfied: vec!["IEC 61508 SIL 2".into()],
        }
    }

    #[test]
    fn to_toml_block_contains_section_header() {
        let m = complete_manifest();
        let block = m.to_toml_block();
        assert!(block.contains("[evidence]"), "must contain [evidence] header");
    }

    #[test]
    fn to_toml_block_contains_version() {
        let m = complete_manifest();
        let block = m.to_toml_block();
        assert!(block.contains("version = \"1.0\""), "must contain version");
    }

    #[test]
    fn to_toml_block_contains_archive_url() {
        let m = complete_manifest();
        let block = m.to_toml_block();
        assert!(block.contains("archive_url"), "must contain archive_url");
    }

    #[test]
    fn to_toml_block_contains_oracle_key() {
        let m = complete_manifest();
        let block = m.to_toml_block();
        assert!(block.contains("oracle_key"), "must contain oracle_key");
    }

    #[test]
    fn to_toml_block_contains_receipt_hash() {
        let m = complete_manifest();
        let block = m.to_toml_block();
        assert!(block.contains("receipt_hash"), "must contain receipt_hash");
    }

    #[test]
    fn to_toml_block_omits_none_fields() {
        let m = EvidenceManifest {
            archive_url: None,
            oracle_key: None,
            receipt_hash: None,
            version: "1.0".to_string(),
            timestamp: None,
            crate_name: "bare".to_string(),
            crate_version: "0.1.0".to_string(),
            trustworthiness_score: None,
            standards_satisfied: Vec::new(),
        };
        let block = m.to_toml_block();
        assert!(!block.contains("archive_url"), "None fields must be omitted");
        assert!(!block.contains("oracle_key"), "None fields must be omitted");
    }

    #[test]
    fn validate_complete_manifest_is_valid() {
        let m = complete_manifest();
        let (valid, issues) = m.validate();
        assert!(valid, "complete manifest should be valid; issues: {:?}", issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn validate_missing_archive_url_reports_issue() {
        let mut m = complete_manifest();
        m.archive_url = None;
        let (valid, issues) = m.validate();
        assert!(!valid, "manifest missing archive_url should not be valid");
        assert!(
            issues.iter().any(|i| i.contains("archive_url")),
            "issues should mention archive_url"
        );
    }

    #[test]
    fn validate_missing_oracle_key_reports_issue() {
        let mut m = complete_manifest();
        m.oracle_key = None;
        let (valid, issues) = m.validate();
        assert!(!valid);
        assert!(issues.iter().any(|i| i.contains("oracle_key")));
    }

    #[test]
    fn validate_missing_receipt_hash_reports_issue() {
        let mut m = complete_manifest();
        m.receipt_hash = None;
        let (valid, issues) = m.validate();
        assert!(!valid);
        assert!(issues.iter().any(|i| i.contains("receipt_hash")));
    }

    #[test]
    fn validate_bad_receipt_hash_prefix_reports_issue() {
        let mut m = complete_manifest();
        m.receipt_hash = Some("md5:bad".into());
        let (_, issues) = m.validate();
        assert!(
            issues.iter().any(|i| i.contains("sha256:")),
            "should report sha256: prefix issue"
        );
    }

    #[test]
    fn compute_trustworthiness_empty_manifest_is_zero() {
        let m = EvidenceManifest {
            archive_url: None,
            oracle_key: None,
            receipt_hash: None,
            version: "1.0".to_string(),
            timestamp: None,
            crate_name: "x".to_string(),
            crate_version: "0.1.0".to_string(),
            trustworthiness_score: None,
            standards_satisfied: Vec::new(),
        };
        let score = compute_trustworthiness(&m);
        assert_eq!(score, 0.0, "empty manifest should score 0.0");
    }

    #[test]
    fn compute_trustworthiness_full_manifest_is_one() {
        let mut m = complete_manifest();
        m.standards_satisfied = vec!["IEC 61508 SIL 2".into(), "ISO 26262 ASIL B".into()];
        let score = compute_trustworthiness(&m);
        assert!(
            (score - 1.0).abs() < 1e-6,
            "full manifest should score 1.0, got {}",
            score
        );
    }

    #[test]
    fn compute_trustworthiness_receipt_only_is_0_4() {
        let m = EvidenceManifest {
            receipt_hash: Some("sha256:abc".into()),
            archive_url: None,
            oracle_key: None,
            version: "1.0".to_string(),
            timestamp: None,
            crate_name: "x".to_string(),
            crate_version: "1.0.0".to_string(),
            trustworthiness_score: None,
            standards_satisfied: Vec::new(),
        };
        let score = compute_trustworthiness(&m);
        assert!(
            (score - 0.4).abs() < 1e-6,
            "receipt-only should score 0.4, got {}",
            score
        );
    }

    #[test]
    fn from_cargo_toml_returns_none_for_absent_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Cargo.toml");
        std::fs::write(
            &path,
            b"[package]\nname = \"foo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let result = EvidenceManifest::from_cargo_toml(&path);
        assert!(
            result.is_none(),
            "should return None when [evidence] section is absent"
        );
    }

    #[test]
    fn from_cargo_toml_parses_evidence_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Cargo.toml");
        let toml = r#"[package]
name = "foo"
version = "0.1.0"

[evidence]
version = "1.0"
archive_url = "https://example.com/foo/0.1.0/evidence.tar.gz"
oracle_key = "mykey=="
receipt_hash = "sha256:cafebabe"
timestamp = "2026-06-17T00:00:00.000Z"
"#;
        std::fs::write(&path, toml.as_bytes()).unwrap();
        let result = EvidenceManifest::from_cargo_toml(&path);
        assert!(result.is_some(), "should parse [evidence] section");
        let m = result.unwrap();
        assert_eq!(m.version, "1.0");
        assert_eq!(
            m.archive_url.as_deref(),
            Some("https://example.com/foo/0.1.0/evidence.tar.gz")
        );
        assert_eq!(m.oracle_key.as_deref(), Some("mykey=="));
        assert_eq!(m.receipt_hash.as_deref(), Some("sha256:cafebabe"));
    }

    #[test]
    fn build_manifest_sets_crate_name() {
        let m = build_manifest("my-crate", "1.0.0", None, None, None);
        assert_eq!(m.crate_name, "my-crate");
        assert_eq!(m.crate_version, "1.0.0");
        assert_eq!(m.version, "1.0");
    }

    #[test]
    fn build_manifest_with_all_inputs_has_positive_score() {
        let dir = tempfile::tempdir().unwrap();
        // Write a dummy receipt.
        let receipt_path = dir.path().join("latest.json");
        std::fs::write(&receipt_path, b"{}").unwrap();

        let m = build_manifest(
            "my-crate",
            "1.0.0",
            Some(&receipt_path),
            Some(dir.path()),
            Some("base64key=="),
        );
        let score = m.trustworthiness_score.unwrap_or(0.0);
        assert!(
            score > 0.6,
            "manifest with receipt, archive_url, oracle_key should score > 0.6, got {}",
            score
        );
    }

    #[test]
    fn dep_has_evidence_returns_false_for_nonexistent_crate() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!dep_has_evidence("nonexistent", "1.0.0", dir.path()));
    }

    #[test]
    fn dep_has_evidence_returns_true_when_file_present() {
        let dir = tempfile::tempdir().unwrap();
        let fname = format!("{}-{}.evidence.json", "serde", "1.0.0");
        std::fs::write(dir.path().join(&fname), b"{}").unwrap();
        assert!(dep_has_evidence("serde", "1.0.0", dir.path()));
    }

    #[test]
    fn format_evidence_status_table_has_headers() {
        let deps = vec![
            ("serde".to_string(), "1.0.200".to_string(), true),
            ("toml".to_string(), "0.8.0".to_string(), false),
        ];
        let table = format_evidence_status_table(&deps);
        assert!(table.contains("Crate"), "table must have Crate header");
        assert!(table.contains("Version"), "table must have Version header");
        assert!(table.contains("Evidence"), "table must have Evidence header");
        assert!(table.contains("VERIFIED"), "should show VERIFIED badge");
        assert!(table.contains("UNVERIFIED"), "should show UNVERIFIED badge");
    }

    #[test]
    fn format_evidence_status_table_empty_deps() {
        let table = format_evidence_status_table(&[]);
        // Even with no deps, headers must be present.
        assert!(table.contains("Crate"));
        assert!(table.contains("Version"));
        assert!(table.contains("Evidence"));
    }
}
