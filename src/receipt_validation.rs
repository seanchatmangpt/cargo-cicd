//! Receipt hash validation for wasm4pm-issued receipts.
//!
//! The wasm4pm oracle issues receipts as JSON files. cargo-cicd validates
//! these receipts by:
//!
//! 1. Verifying the file exists on disk.
//! 2. Computing the file's hash and comparing to the expected value.
//! 3. Parsing the JSON to extract the verdict, timestamp, and oracle identifier.
//! 4. Checking that all required fields are present for acceptance.
//!
//! ## Hash implementation
//!
//! Neither `sha2`, `blake3`, nor `ring` are in the current Cargo.toml.
//! This module uses the same FNV-1a fan-out hash used throughout the codebase
//! (`evidence::simple_hex_hash`) and clearly labels it as a stub. To upgrade
//! to true SHA-256, add `sha2 = "0.10"` to Cargo.toml and replace the
//! `sha256_file_hex` body with a `sha2::Sha256` digest.

use std::path::Path;

/// Outcome of validating a receipt file.
#[derive(Debug, PartialEq)]
pub enum ReceiptValidationResult {
    /// Receipt exists, hash matches, and all required fields are present.
    Valid,
    /// Receipt file does not exist at the given path.
    NotFound,
    /// The file's computed hash does not match the expected hash.
    HashMismatch {
        /// Hash value that was expected (from Cargo.toml or caller).
        expected: String,
        /// Hash value that was computed from the file on disk.
        actual: String,
    },
    /// The file content could not be parsed as JSON.
    ParseError(String),
    /// A required JSON field is absent from the receipt.
    MissingField(String),
}

/// Required top-level fields in a wasm4pm receipt JSON.
const REQUIRED_RECEIPT_FIELDS: &[&str] =
    &["verdict", "oracle_id", "timestamp", "case_id", "trace_hash"];

/// Validate the receipt file at `receipt_path` against `expected_hash`.
///
/// Validation steps (in order):
/// 1. File must exist → [`ReceiptValidationResult::NotFound`] otherwise.
/// 2. Computed hash must equal `expected_hash` → [`ReceiptValidationResult::HashMismatch`] otherwise.
/// 3. JSON must be parseable → [`ReceiptValidationResult::ParseError`] otherwise.
/// 4. All required fields must be present → [`ReceiptValidationResult::MissingField`] otherwise.
/// 5. All checks pass → [`ReceiptValidationResult::Valid`].
pub fn validate_receipt_file(receipt_path: &Path, expected_hash: &str) -> ReceiptValidationResult {
    if !receipt_path.exists() {
        return ReceiptValidationResult::NotFound;
    }

    let actual_hash = match sha256_file_hex(receipt_path) {
        Ok(h) => h,
        Err(e) => return ReceiptValidationResult::ParseError(format!("hash error: {}", e)),
    };

    if actual_hash != expected_hash {
        return ReceiptValidationResult::HashMismatch {
            expected: expected_hash.to_string(),
            actual: actual_hash,
        };
    }

    let content = match std::fs::read_to_string(receipt_path) {
        Ok(s) => s,
        Err(e) => return ReceiptValidationResult::ParseError(e.to_string()),
    };

    let missing = receipt_has_required_fields(&content);
    if let Some(field) = missing.into_iter().next() {
        return ReceiptValidationResult::MissingField(field);
    }

    ReceiptValidationResult::Valid
}

/// Compute a hash of the file at `path` and return it as a lowercase hex string.
///
/// # Implementation note
///
/// This is a **stub implementation** using FNV-1a fan-out hashing (the same
/// algorithm used in `evidence::simple_hex_hash`). It produces a 64-character
/// hex string that is stable and consistent, but is **not** a true SHA-256 hash.
///
/// For a real cryptographic digest, run the receipt through the affidavit
/// provenance engine (`cargo cicd affidavit verify`), which BLAKE3-content-
/// addresses the receipt out-of-process via the `affi` CLI.
pub fn sha256_file_hex(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    // STUB: FNV-1a fan-out hash. Real BLAKE3 is provided by the affi oracle.
    Ok(fnv1a_fan_out_hex(&bytes))
}

/// Parse a receipt JSON string and extract the primary verdict fields.
///
/// Returns `Ok((verdict, timestamp, oracle_id))` on success, or `Err` with a
/// description of what failed.
///
/// # Expected JSON shape
///
/// ```json
/// {
///   "verdict": "Accept",
///   "timestamp": "2026-06-17T00:00:00Z",
///   "oracle_id": "wasm4pm/v26.5.29"
/// }
/// ```
pub fn parse_receipt_json(json_str: &str) -> Result<(String, String, String), String> {
    let value: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {}", e))?;

    let verdict = value
        .get("verdict")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing field: verdict".to_string())?
        .to_string();

    let timestamp = value
        .get("timestamp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing field: timestamp".to_string())?
        .to_string();

    let oracle_id = value
        .get("oracle_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing field: oracle_id".to_string())?
        .to_string();

    Ok((verdict, timestamp, oracle_id))
}

/// Check which required fields are missing from a receipt JSON string.
///
/// Returns the names of any absent required fields. Returns an empty `Vec`
/// if all required fields are present (even if their values are null).
///
/// Required fields: `"verdict"`, `"oracle_id"`, `"timestamp"`, `"case_id"`,
/// `"trace_hash"`.
pub fn receipt_has_required_fields(json_str: &str) -> Vec<String> {
    let value: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return REQUIRED_RECEIPT_FIELDS.iter().map(|s| s.to_string()).collect(),
    };

    REQUIRED_RECEIPT_FIELDS
        .iter()
        .filter(|&&field| value.get(field).is_none())
        .map(|&f| f.to_string())
        .collect()
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// FNV-1a fan-out hash producing a 64-char lowercase hex string.
///
/// Identical to `evidence::simple_hex_hash`. Duplicated here to keep the
/// receipt_validation module self-contained without a cross-module dependency.
fn fnv1a_fan_out_hex(data: &[u8]) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_file(content: &str) -> (tempfile::NamedTempFile, String) {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.flush().unwrap();
        let hash = sha256_file_hex(f.path()).unwrap();
        (f, hash)
    }

    #[test]
    fn sha256_file_hex_returns_64_char_hex() {
        let (f, hash) = write_temp_file("hello world");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        drop(f);
    }

    #[test]
    fn sha256_file_hex_returns_err_for_missing_file() {
        let result = sha256_file_hex(Path::new("/nonexistent/path/to/file.json"));
        assert!(result.is_err());
    }

    #[test]
    fn validate_receipt_file_not_found() {
        let result =
            validate_receipt_file(Path::new("/nonexistent/receipt.json"), "expected_hash");
        assert_eq!(result, ReceiptValidationResult::NotFound);
    }

    #[test]
    fn validate_receipt_file_hash_mismatch() {
        let (f, _real_hash) = write_temp_file(r#"{"verdict":"Accept","oracle_id":"wasm4pm","timestamp":"2026-01-01T00:00:00Z","case_id":"c1","trace_hash":"th1"}"#);
        let result = validate_receipt_file(f.path(), "wrong_hash_value");
        assert!(matches!(result, ReceiptValidationResult::HashMismatch { .. }));
    }

    #[test]
    fn parse_receipt_json_valid() {
        let json = r#"{"verdict":"Accept","oracle_id":"wasm4pm/v1","timestamp":"2026-06-17T00:00:00Z"}"#;
        let (verdict, ts, oracle) = parse_receipt_json(json).unwrap();
        assert_eq!(verdict, "Accept");
        assert_eq!(ts, "2026-06-17T00:00:00Z");
        assert_eq!(oracle, "wasm4pm/v1");
    }

    #[test]
    fn parse_receipt_json_malformed() {
        let result = parse_receipt_json("not json at all {{{");
        assert!(result.is_err());
    }

    #[test]
    fn receipt_has_required_fields_all_present() {
        let json = r#"{"verdict":"Accept","oracle_id":"x","timestamp":"t","case_id":"c","trace_hash":"h"}"#;
        let missing = receipt_has_required_fields(json);
        assert!(missing.is_empty(), "expected no missing fields, got: {:?}", missing);
    }

    #[test]
    fn receipt_has_required_fields_returns_missing() {
        let json = r#"{"verdict":"Accept"}"#;
        let missing = receipt_has_required_fields(json);
        assert!(missing.contains(&"oracle_id".to_string()));
        assert!(missing.contains(&"timestamp".to_string()));
        assert!(missing.contains(&"case_id".to_string()));
        assert!(missing.contains(&"trace_hash".to_string()));
    }
}
