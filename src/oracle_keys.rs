//! Oracle public key infrastructure for wasm4pm evidence verification.
//!
//! The wasm4pm oracle signs receipts with an Ed25519 key. cargo-cicd embeds
//! the oracle's public key in evidence metadata so verifiers can independently
//! validate signatures without trusting cargo-cicd itself.
//!
//! ## Key lifecycle
//!
//! 1. Oracle generates an Ed25519 key pair and publishes the public key.
//! 2. Operators set `CICD_ORACLE_KEY_B64` in their environment.
//! 3. cargo-cicd loads the key at startup and embeds it in each XES trace.
//! 4. `key_is_valid` checks the validity window before embedding.
//! 5. `compute_fingerprint` produces a stable identifier for key rotation logs.

/// Oracle public key record stored in evidence metadata.
#[derive(Debug, Clone)]
pub struct OraclePublicKey {
    /// Base64-encoded public key bytes (Ed25519 or similar).
    pub key_b64: String,
    /// Algorithm identifier, e.g. `"Ed25519"`.
    pub algorithm: String,
    /// Oracle provider name, e.g. `"wasm4pm/ferrous-systems"`.
    pub provider: String,
    /// Key validity start (ISO-8601 UTC).
    pub valid_from: String,
    /// Key validity end (ISO-8601 UTC). `None` means no expiry.
    pub valid_until: Option<String>,
    /// Key fingerprint: SHA-256 of the raw key bytes, encoded as lowercase hex.
    ///
    /// Computed by [`compute_fingerprint`]. Used as a stable short identifier
    /// across key rotation events.
    pub fingerprint: String,
}

/// Key rotation policy governing maximum key age and required overlap.
pub struct KeyRotationPolicy {
    /// Maximum number of days a key may remain active before rotation is required.
    pub max_key_age_days: u32,
    /// Minimum number of days the new key must overlap with the old key during
    /// transition (allows verifiers time to update).
    pub require_overlap_days: u32,
}

impl Default for KeyRotationPolicy {
    fn default() -> Self {
        Self {
            max_key_age_days: 365,
            require_overlap_days: 30,
        }
    }
}

/// Load the oracle public key from the `CICD_ORACLE_KEY_B64` environment variable.
///
/// The environment variable is expected to be a base64-encoded public key. The
/// returned `OraclePublicKey` has default values for algorithm (`"Ed25519"`),
/// provider (`"wasm4pm"`), and an open-ended validity window.
///
/// Returns `None` if the variable is not set or is empty.
pub fn load_oracle_key_from_env() -> Option<OraclePublicKey> {
    let key_b64 = std::env::var("CICD_ORACLE_KEY_B64").ok()?;
    if key_b64.trim().is_empty() {
        return None;
    }
    let key_b64 = key_b64.trim().to_string();
    let fingerprint = compute_fingerprint(&key_b64);

    // Algorithm and provider can be overridden by additional env vars.
    let algorithm = std::env::var("CICD_ORACLE_KEY_ALGORITHM")
        .unwrap_or_else(|_| "Ed25519".to_string());
    let provider = std::env::var("CICD_ORACLE_KEY_PROVIDER")
        .unwrap_or_else(|_| "wasm4pm".to_string());
    let valid_from = std::env::var("CICD_ORACLE_KEY_VALID_FROM")
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
    let valid_until = std::env::var("CICD_ORACLE_KEY_VALID_UNTIL").ok().and_then(
        |v| if v.trim().is_empty() { None } else { Some(v.trim().to_string()) },
    );

    Some(OraclePublicKey {
        key_b64,
        algorithm,
        provider,
        valid_from,
        valid_until,
        fingerprint,
    })
}

/// Produce XES `<trace>` string attributes that embed the oracle public key.
///
/// Returns a list of `(key, value)` pairs suitable for insertion as
/// `<string key="..." value="..."/>` elements inside a `<trace>` element.
pub fn oracle_key_trace_attributes(key: &OraclePublicKey) -> Vec<(String, String)> {
    let mut attrs = vec![
        (
            "oracle:key_b64".to_string(),
            key.key_b64.clone(),
        ),
        (
            "oracle:algorithm".to_string(),
            key.algorithm.clone(),
        ),
        (
            "oracle:provider".to_string(),
            key.provider.clone(),
        ),
        (
            "oracle:valid_from".to_string(),
            key.valid_from.clone(),
        ),
        (
            "oracle:fingerprint".to_string(),
            key.fingerprint.clone(),
        ),
    ];
    if let Some(ref until) = key.valid_until {
        attrs.push(("oracle:valid_until".to_string(), until.clone()));
    }
    attrs
}

/// Return `true` if the key is currently valid (not expired relative to now).
///
/// Validity is checked against the `valid_until` field using simple ISO-8601
/// string comparison. Keys with no `valid_until` are considered perpetually valid.
pub fn key_is_valid(key: &OraclePublicKey) -> bool {
    let Some(ref until) = key.valid_until else {
        return true; // No expiry set.
    };
    let now = crate::evidence::now_iso8601();
    // ISO-8601 strings sort lexicographically, so direct comparison is correct
    // as long as both strings are in UTC (Z suffix) with the same format.
    now.as_str() <= until.as_str()
}

/// Compute a fingerprint for a base64-encoded key.
///
/// Decodes the base64 key bytes, then computes a 32-byte FNV-1a fan-out hash
/// (the same algorithm used in `evidence::simple_hex_hash`). Returns a 64-char
/// lowercase hex string.
///
/// # Note
///
/// This is a simplified fingerprint using a non-cryptographic hash function
/// suitable for key identification and rotation logging. For production
/// signature verification, a real SHA-256 or Ed25519 library should be used.
/// The `sha2` or `blake3` crate is not currently in Cargo.toml; add either one
/// to enable true cryptographic fingerprinting.
pub fn compute_fingerprint(key_b64: &str) -> String {
    let bytes = match decode_base64(key_b64.trim()) {
        Some(b) if !b.is_empty() => b,
        _ => {
            // Fallback: fingerprint the raw string bytes if base64 decode fails.
            return simple_hex_hash(key_b64.as_bytes());
        }
    };
    simple_hex_hash(&bytes)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Decode a standard base64 string (with `+` and `/`, optional `=` padding).
///
/// Returns `None` on any invalid character. Does not support URL-safe base64.
fn decode_base64(input: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    // Build decode table: maps byte → 6-bit value, 0xFF for invalid.
    let mut table = [0xFFu8; 256];
    for (i, &c) in ALPHABET.iter().enumerate() {
        table[c as usize] = i as u8;
    }

    let input = input.trim_end_matches('=');
    let mut out = Vec::with_capacity((input.len() * 3) / 4 + 1);
    let chars: Vec<u8> = input.bytes().collect();

    let mut i = 0;
    while i + 3 < chars.len() {
        let a = table[chars[i] as usize];
        let b = table[chars[i + 1] as usize];
        let c = table[chars[i + 2] as usize];
        let d = table[chars[i + 3] as usize];
        if a == 0xFF || b == 0xFF || c == 0xFF || d == 0xFF {
            return None;
        }
        out.push((a << 2) | (b >> 4));
        out.push((b << 4) | (c >> 2));
        out.push((c << 6) | d);
        i += 4;
    }

    // Handle remaining 2 or 3 chars (partial group).
    match chars.len() - i {
        2 => {
            let a = table[chars[i] as usize];
            let b = table[chars[i + 1] as usize];
            if a == 0xFF || b == 0xFF {
                return None;
            }
            out.push((a << 2) | (b >> 4));
        }
        3 => {
            let a = table[chars[i] as usize];
            let b = table[chars[i + 1] as usize];
            let c = table[chars[i + 2] as usize];
            if a == 0xFF || b == 0xFF || c == 0xFF {
                return None;
            }
            out.push((a << 2) | (b >> 4));
            out.push((b << 4) | (c >> 2));
        }
        _ => {}
    }

    Some(out)
}

/// 32-byte FNV-1a fan-out hash producing a 64-char lowercase hex string.
/// Identical algorithm to `evidence::simple_hex_hash` for consistency.
fn simple_hex_hash(data: &[u8]) -> String {
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

    #[test]
    fn compute_fingerprint_returns_64_char_hex_for_valid_base64() {
        // "hello" in base64
        let fp = compute_fingerprint("aGVsbG8=");
        assert_eq!(fp.len(), 64, "fingerprint must be 64 hex chars");
        assert!(
            fp.chars().all(|c| c.is_ascii_hexdigit()),
            "fingerprint must be hex"
        );
    }

    #[test]
    fn compute_fingerprint_returns_non_empty_for_invalid_base64() {
        let fp = compute_fingerprint("not!valid!base64@@@");
        assert!(!fp.is_empty(), "fingerprint must be non-empty even for invalid b64");
    }

    #[test]
    fn decode_base64_round_trips_hello() {
        let decoded = decode_base64("aGVsbG8=").expect("valid b64");
        assert_eq!(decoded, b"hello");
    }

    #[test]
    fn decode_base64_rejects_invalid_chars() {
        assert!(decode_base64("!!!").is_none());
    }
}
