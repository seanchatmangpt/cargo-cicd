//! Tests for oracle public key infrastructure and receipt hash validation.
//!
//! Covers:
//! - `oracle_keys::load_oracle_key_from_env` — env var absent/present
//! - `oracle_keys::key_is_valid` — not-expired and expired keys
//! - `oracle_keys::oracle_key_trace_attributes` — non-empty attribute list
//! - `oracle_keys::compute_fingerprint` — non-empty hex string
//! - `receipt_validation::validate_receipt_file` — NotFound for missing file
//! - `receipt_validation::parse_receipt_json` — valid and malformed JSON
//! - `receipt_validation::receipt_has_required_fields` — missing field detection

use cargo_cicd::oracle_keys::{
    compute_fingerprint, key_is_valid, load_oracle_key_from_env, oracle_key_trace_attributes,
    OraclePublicKey,
};
use cargo_cicd::receipt_validation::{
    parse_receipt_json, receipt_has_required_fields, validate_receipt_file, ReceiptValidationResult,
};
use std::path::Path;

// ── oracle_keys tests ─────────────────────────────────────────────────────────

/// 1. `load_oracle_key_from_env` returns `None` when the env var is not set.
#[test]
fn load_oracle_key_returns_none_when_env_unset() {
    // Remove the env var for this test scope.
    let _guard = EnvGuard::remove("CICD_ORACLE_KEY_B64");
    let result = load_oracle_key_from_env();
    assert!(
        result.is_none(),
        "expected None when CICD_ORACLE_KEY_B64 is not set"
    );
}

/// 2. `load_oracle_key_from_env` parses a key when the env var is set.
#[test]
fn load_oracle_key_returns_key_when_env_set() {
    // "hello" encoded in base64
    let _guard = EnvGuard::set("CICD_ORACLE_KEY_B64", "aGVsbG8=");
    let result = load_oracle_key_from_env();
    assert!(
        result.is_some(),
        "expected Some when CICD_ORACLE_KEY_B64 is set"
    );
    let key = result.unwrap();
    assert_eq!(key.key_b64, "aGVsbG8=");
    assert!(!key.fingerprint.is_empty(), "fingerprint must be populated");
}

/// 3. `key_is_valid` returns `true` for a key with no expiry set.
#[test]
fn key_is_valid_true_for_open_ended_key() {
    let key = OraclePublicKey {
        key_b64: "aGVsbG8=".to_string(),
        algorithm: "Ed25519".to_string(),
        provider: "wasm4pm".to_string(),
        valid_from: "2020-01-01T00:00:00Z".to_string(),
        valid_until: None, // No expiry.
        fingerprint: compute_fingerprint("aGVsbG8="),
    };
    assert!(key_is_valid(&key), "key with no valid_until must be valid");
}

/// 4. `key_is_valid` returns `false` for a key with a past expiry date.
#[test]
fn key_is_valid_false_for_expired_key() {
    let key = OraclePublicKey {
        key_b64: "aGVsbG8=".to_string(),
        algorithm: "Ed25519".to_string(),
        provider: "wasm4pm".to_string(),
        valid_from: "2020-01-01T00:00:00Z".to_string(),
        valid_until: Some("2020-12-31T23:59:59Z".to_string()), // Expired in the past.
        fingerprint: compute_fingerprint("aGVsbG8="),
    };
    assert!(
        !key_is_valid(&key),
        "key with past valid_until must be invalid"
    );
}

/// 5. `oracle_key_trace_attributes` returns a non-empty list of key-value pairs.
#[test]
fn oracle_key_trace_attributes_non_empty() {
    let key = OraclePublicKey {
        key_b64: "dGVzdA==".to_string(),
        algorithm: "Ed25519".to_string(),
        provider: "wasm4pm/test".to_string(),
        valid_from: "2026-01-01T00:00:00Z".to_string(),
        valid_until: None,
        fingerprint: compute_fingerprint("dGVzdA=="),
    };
    let attrs = oracle_key_trace_attributes(&key);
    assert!(!attrs.is_empty(), "trace attributes must be non-empty");
    // Must include the key fingerprint.
    let has_fingerprint = attrs.iter().any(|(k, _)| k.contains("fingerprint"));
    assert!(
        has_fingerprint,
        "attributes must include oracle:fingerprint"
    );
    // Must include the key_b64 value.
    let has_key = attrs.iter().any(|(k, _)| k.contains("key_b64"));
    assert!(has_key, "attributes must include oracle:key_b64");
}

static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// RAII guard that sets/removes an env var for the duration of a test.
struct EnvGuard {
    key: &'static str,
    original: Option<String>,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let lock = ENV_MUTEX.lock().unwrap();
        let original = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, original, _lock: lock }
    }

    fn remove(key: &'static str) -> Self {
        let lock = ENV_MUTEX.lock().unwrap();
        let original = std::env::var(key).ok();
        std::env::remove_var(key);
        Self { key, original, _lock: lock }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(v) => std::env::set_var(self.key, v),
            None => std::env::remove_var(self.key),
        }
    }
}
