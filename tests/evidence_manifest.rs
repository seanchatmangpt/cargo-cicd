//! Integration tests for the `evidence_manifest` module.
//!
//! These tests exercise `build_manifest`, `to_toml_block`, `from_cargo_toml`,
//! `compute_trustworthiness`, `validate`, `dep_has_evidence`, and
//! `format_evidence_status_table` to ensure the Vision 2030 evidence metadata
//! generation pipeline is correct end-to-end.

use cargo_cicd::evidence_manifest::{
    build_manifest, compute_trustworthiness, dep_has_evidence, format_evidence_status_table,
    EvidenceManifest,
};
use tempfile::TempDir;

// ─── Helper ───────────────────────────────────────────────────────────────────

fn full_manifest() -> EvidenceManifest {
    EvidenceManifest {
        archive_url: Some("https://evidence.cargo-cicd.rs/my-crate/1.0.0/evidence.tar.gz".into()),
        oracle_key: Some("base64key==".into()),
        receipt_hash: Some("sha256:deadbeefcafe".into()),
        version: "1.0".to_string(),
        timestamp: Some("2026-06-17T00:00:00.000Z".into()),
        crate_name: "my-crate".to_string(),
        crate_version: "1.0.0".to_string(),
        trustworthiness_score: Some(1.0),
        standards_satisfied: vec!["IEC 61508 SIL 2".into(), "ISO 26262 ASIL B".into()],
    }
}

fn empty_manifest() -> EvidenceManifest {
    EvidenceManifest {
        archive_url: None,
        oracle_key: None,
        receipt_hash: None,
        version: "1.0".to_string(),
        timestamp: None,
        crate_name: "bare-crate".to_string(),
        crate_version: "0.1.0".to_string(),
        trustworthiness_score: None,
        standards_satisfied: Vec::new(),
    }
}

// ─── Test 1: build_manifest sets crate_name ──────────────────────────────────

#[test]
fn test_build_manifest_sets_crate_name() {
    let m = build_manifest("my-crate", "1.0.0", None, None, None);
    assert_eq!(
        m.crate_name, "my-crate",
        "build_manifest must set crate_name"
    );
    assert_eq!(
        m.crate_version, "1.0.0",
        "build_manifest must set crate_version"
    );
    assert_eq!(m.version, "1.0", "default schema version must be '1.0'");
}

// ─── Test 2: to_toml_block contains [evidence] header ────────────────────────

#[test]
fn test_to_toml_block_contains_evidence_header() {
    let m = full_manifest();
    let block = m.to_toml_block();
    assert!(
        block.contains("[evidence]"),
        "to_toml_block must start with [evidence] section header; got:\n{}",
        block
    );
}

// ─── Test 3: to_toml_block contains version = "1.0" ─────────────────────────

#[test]
fn test_to_toml_block_contains_version() {
    let m = full_manifest();
    let block = m.to_toml_block();
    assert!(
        block.contains("version = \"1.0\""),
        "to_toml_block must contain version = \"1.0\"; got:\n{}",
        block
    );
}

// ─── Test 4: compute_trustworthiness returns 0.0 for empty manifest ──────────

#[test]
fn test_compute_trustworthiness_empty_is_zero() {
    let m = empty_manifest();
    let score = compute_trustworthiness(&m);
    assert_eq!(
        score, 0.0,
        "empty manifest (no fields set) must score 0.0, got {}",
        score
    );
}

// ─── Test 5: compute_trustworthiness > 0.6 with receipt + archive + key ──────

#[test]
fn test_compute_trustworthiness_high_when_three_fields_present() {
    let m = EvidenceManifest {
        archive_url: Some("https://example.com/evidence.tar.gz".into()),
        oracle_key: Some("base64key==".into()),
        receipt_hash: Some("sha256:cafebabe".into()),
        version: "1.0".to_string(),
        timestamp: None,
        crate_name: "my-crate".to_string(),
        crate_version: "1.0.0".to_string(),
        trustworthiness_score: None,
        standards_satisfied: Vec::new(),
    };
    let score = compute_trustworthiness(&m);
    assert!(
        score > 0.6,
        "manifest with receipt (0.4) + archive (0.2) + oracle_key (0.2) should score > 0.6, got {}",
        score
    );
    // Exact expected: 0.4 + 0.2 + 0.2 = 0.8
    assert!((score - 0.8).abs() < 1e-5, "expected 0.8, got {}", score);
}

// ─── Test 6: validate returns issues for missing archive_url ─────────────────

#[test]
fn test_validate_missing_archive_url_returns_issue() {
    let mut m = full_manifest();
    m.archive_url = None;
    let (valid, issues) = m.validate();
    assert!(
        !valid,
        "manifest without archive_url should fail validation"
    );
    assert!(
        issues.iter().any(|i| i.contains("archive_url")),
        "issues should mention 'archive_url'; got: {:?}",
        issues
    );
}

// ─── Test 7: validate returns (true, []) for complete manifest ───────────────

#[test]
fn test_validate_complete_manifest_is_valid() {
    let m = full_manifest();
    let (valid, issues) = m.validate();
    assert!(
        valid,
        "complete manifest should be valid; issues: {:?}",
        issues
    );
    assert!(
        issues.is_empty(),
        "complete manifest should have no validation issues; got: {:?}",
        issues
    );
}

// ─── Test 8: dep_has_evidence returns false for nonexistent crate ────────────

#[test]
fn test_dep_has_evidence_nonexistent_returns_false() {
    let dir = TempDir::new().expect("tempdir");
    let result = dep_has_evidence("nonexistent", "1.0.0", dir.path());
    assert!(
        !result,
        "dep_has_evidence should return false for a crate with no evidence file"
    );
}

// ─── Test 9: format_evidence_status_table returns table with headers ─────────

#[test]
fn test_format_evidence_status_table_has_headers() {
    let deps = vec![
        ("serde".to_string(), "1.0.200".to_string(), true),
        ("toml".to_string(), "0.8.0".to_string(), false),
    ];
    let table = format_evidence_status_table(&deps);

    assert!(
        table.contains("Crate"),
        "table must have 'Crate' column header; got:\n{}",
        table
    );
    assert!(
        table.contains("Version"),
        "table must have 'Version' column header; got:\n{}",
        table
    );
    assert!(
        table.contains("Evidence"),
        "table must have 'Evidence' column header; got:\n{}",
        table
    );
}

// ─── Test 10: from_cargo_toml returns None for Cargo.toml without [evidence] ─

#[test]
fn test_from_cargo_toml_returns_none_when_section_absent() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("Cargo.toml");

    let content = r#"[package]
name = "my-crate"
version = "1.0.0"
edition = "2021"

[dependencies]
serde = "1"
"#;
    std::fs::write(&path, content.as_bytes()).expect("write Cargo.toml");

    let result = EvidenceManifest::from_cargo_toml(&path);
    assert!(
        result.is_none(),
        "from_cargo_toml must return None when [evidence] section is absent"
    );
}

// ─── Bonus tests ──────────────────────────────────────────────────────────────

#[test]
fn test_dep_has_evidence_returns_true_when_json_file_exists() {
    let dir = TempDir::new().expect("tempdir");
    let fname = "anyhow-1.0.75.evidence.json";
    std::fs::write(dir.path().join(fname), b"{}").expect("write evidence file");
    assert!(
        dep_has_evidence("anyhow", "1.0.75", dir.path()),
        "should return true when .evidence.json file exists"
    );
}

#[test]
fn test_dep_has_evidence_returns_true_when_toml_file_exists() {
    let dir = TempDir::new().expect("tempdir");
    let fname = "walkdir-2.4.0.evidence.toml";
    std::fs::write(dir.path().join(fname), b"[evidence]\nversion = \"1.0\"\n").expect("write");
    assert!(dep_has_evidence("walkdir", "2.4.0", dir.path()));
}

#[test]
fn test_dep_has_evidence_returns_false_for_missing_cache_dir() {
    let nonexistent = std::path::PathBuf::from("/tmp/cargo-cicd-evidence-test-no-such-dir-xyz");
    assert!(!dep_has_evidence("serde", "1.0.0", &nonexistent));
}

#[test]
fn test_format_evidence_status_table_verified_unverified_badges() {
    let deps = vec![
        ("serde".to_string(), "1.0.200".to_string(), true),
        ("toml".to_string(), "0.8.0".to_string(), false),
    ];
    let table = format_evidence_status_table(&deps);
    assert!(
        table.contains("VERIFIED"),
        "table must show VERIFIED badge for evidenced crate"
    );
    assert!(
        table.contains("UNVERIFIED"),
        "table must show UNVERIFIED badge for unevidenced crate"
    );
}

#[test]
fn test_to_toml_block_round_trip_via_parse() {
    let dir = TempDir::new().expect("tempdir");
    let cargo_toml_path = dir.path().join("Cargo.toml");

    // Write a Cargo.toml with an evidence section.
    let m = full_manifest();
    let block = m.to_toml_block();
    let full_toml = format!(
        "[package]\nname = \"my-crate\"\nversion = \"1.0.0\"\n\n{}",
        block
    );
    std::fs::write(&cargo_toml_path, full_toml.as_bytes()).expect("write");

    let parsed = EvidenceManifest::from_cargo_toml(&cargo_toml_path);
    assert!(
        parsed.is_some(),
        "from_cargo_toml should parse the written block"
    );
    let parsed = parsed.unwrap();
    assert_eq!(parsed.version, "1.0");
    assert_eq!(
        parsed.archive_url.as_deref(),
        Some("https://evidence.cargo-cicd.rs/my-crate/1.0.0/evidence.tar.gz")
    );
}

#[test]
fn test_build_manifest_with_receipt_file_has_receipt_hash() {
    let dir = TempDir::new().expect("tempdir");
    let receipt_path = dir.path().join("latest.json");
    std::fs::write(&receipt_path, b"{\"receipt_id\":\"test\"}").expect("write receipt");

    let m = build_manifest("my-crate", "1.0.0", Some(&receipt_path), None, None);
    assert!(
        m.receipt_hash.is_some(),
        "build_manifest with receipt_path should set receipt_hash"
    );
    let hash = m.receipt_hash.unwrap();
    assert!(
        hash.starts_with("sha256:"),
        "receipt_hash should have 'sha256:' prefix; got {}",
        hash
    );
}

#[test]
fn test_build_manifest_with_evidence_dir_has_archive_url() {
    let dir = TempDir::new().expect("tempdir");
    let m = build_manifest("my-crate", "2.0.0", None, Some(dir.path()), None);
    assert!(
        m.archive_url.is_some(),
        "build_manifest with evidence_dir should derive archive_url"
    );
    let url = m.archive_url.unwrap();
    assert!(
        url.starts_with("https://"),
        "archive_url should be HTTPS; got {}",
        url
    );
    assert!(
        url.contains("my-crate"),
        "archive_url should include crate name"
    );
    assert!(url.contains("2.0.0"), "archive_url should include version");
}

#[test]
fn test_compute_trustworthiness_full_manifest_scores_one() {
    let mut m = full_manifest();
    m.standards_satisfied = vec!["IEC 61508 SIL 2".into(), "ISO 26262 ASIL B".into()];
    let score = compute_trustworthiness(&m);
    assert!(
        (score - 1.0).abs() < 1e-5,
        "full manifest should score 1.0; got {}",
        score
    );
}

#[test]
fn test_validate_missing_oracle_key_is_invalid() {
    let mut m = full_manifest();
    m.oracle_key = None;
    let (valid, issues) = m.validate();
    assert!(!valid);
    assert!(
        issues.iter().any(|i| i.contains("oracle_key")),
        "should report missing oracle_key; got {:?}",
        issues
    );
}

#[test]
fn test_validate_missing_receipt_hash_is_invalid() {
    let mut m = full_manifest();
    m.receipt_hash = None;
    let (valid, issues) = m.validate();
    assert!(!valid);
    assert!(
        issues.iter().any(|i| i.contains("receipt_hash")),
        "should report missing receipt_hash; got {:?}",
        issues
    );
}

#[test]
fn test_format_evidence_status_table_empty_deps() {
    let table = format_evidence_status_table(&[]);
    assert!(
        table.contains("Crate"),
        "headers must be present even with no deps"
    );
    assert!(
        table.contains("Evidence"),
        "headers must be present even with no deps"
    );
}
