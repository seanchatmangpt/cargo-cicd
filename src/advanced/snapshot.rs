//! Compact binary engine-state snapshots backed by `bitcode`.
//!
//! [`EngineSnapshot`] captures a representative slice of the cargo-cicd
//! process-data engine — workspace identity, toolchain, changed-file set,
//! target footprint, git phase, and per-stage timing records — as a single
//! serializable value.
//!
//! Snapshots are encoded with [`bitcode`], a self-describing-free binary
//! format. Because bitcode omits field names, type tags, and the structural
//! punctuation that JSON repeats on every field, its wire form is far more
//! compact than the equivalent `serde_json` representation of the same
//! snapshot. This matters when many snapshots are persisted or shipped through
//! the engine's event stream, where the encoding overhead dominates payload
//! size. The accompanying test suite asserts this size advantage directly.

use serde::{Deserialize, Serialize};

/// Schema version of the current [`EngineSnapshot`] layout.
///
/// Bump this whenever the on-wire shape of [`EngineSnapshot`] changes so that
/// consumers can detect and reject incompatible binary blobs.
const SCHEMA_VERSION: u32 = 1;

/// A single pipeline-stage outcome captured within an [`EngineSnapshot`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StageRecord {
    /// Human-readable stage name (e.g. `"build"`, `"test"`, `"publish"`).
    pub name: String,
    /// Whether the stage completed successfully.
    pub ok: bool,
    /// Real-time duration of the stage in milliseconds.
    pub elapsed_ms: u64,
}

/// A compact, serializable slice of cargo-cicd engine state.
///
/// Encoded with [`encode`] and recovered with [`decode`]. The binary form is
/// substantially smaller than the JSON form of the same data.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EngineSnapshot {
    /// Absolute path to the workspace root this snapshot describes.
    pub workspace_root: String,
    /// Active toolchain identifier (e.g. `"stable"`, `"1.79.0"`).
    pub toolchain: String,
    /// Paths of files detected as changed relative to the workspace root.
    pub changed_files: Vec<String>,
    /// Total bytes occupied by the `target/` directory.
    pub target_bytes: u64,
    /// Current git phase label (e.g. `"clean"`, `"dirty"`, `"ahead"`).
    pub git_phase: String,
    /// Schema version this snapshot was produced under.
    pub schema_version: u32,
    /// Per-stage timing and outcome records for the captured run.
    pub stages: Vec<StageRecord>,
}

impl EngineSnapshot {
    /// Returns the schema version of the current [`EngineSnapshot`] layout.
    pub fn current_schema_version() -> u32 {
        SCHEMA_VERSION
    }
}

impl Default for EngineSnapshot {
    fn default() -> Self {
        EngineSnapshot {
            workspace_root: String::new(),
            toolchain: String::new(),
            changed_files: Vec::new(),
            target_bytes: 0,
            git_phase: String::new(),
            schema_version: SCHEMA_VERSION,
            stages: Vec::new(),
        }
    }
}

/// Encodes an [`EngineSnapshot`] into a compact bitcode byte buffer.
///
/// Returns an [`std::io::Error`] (kind [`std::io::ErrorKind::Other`]) if the
/// underlying serializer fails.
pub fn encode(snapshot: &EngineSnapshot) -> Result<Vec<u8>, std::io::Error> {
    bitcode::serialize(snapshot).map_err(std::io::Error::other)
}

/// Decodes a bitcode byte buffer produced by [`encode`] back into an
/// [`EngineSnapshot`].
///
/// Returns an [`std::io::Error`] (kind [`std::io::ErrorKind::Other`]) if the
/// bytes are malformed or do not describe a valid snapshot.
pub fn decode(bytes: &[u8]) -> Result<EngineSnapshot, std::io::Error> {
    bitcode::deserialize(bytes).map_err(std::io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> EngineSnapshot {
        EngineSnapshot {
            workspace_root: "/home/user/cargo-cicd".to_string(),
            toolchain: "stable".to_string(),
            changed_files: vec![
                "src/lib.rs".to_string(),
                "src/advanced/snapshot.rs".to_string(),
                "Cargo.toml".to_string(),
            ],
            target_bytes: 1_234_567_890,
            git_phase: "dirty".to_string(),
            schema_version: EngineSnapshot::current_schema_version(),
            stages: vec![
                StageRecord {
                    name: "build".to_string(),
                    ok: true,
                    elapsed_ms: 4_200,
                },
                StageRecord {
                    name: "test".to_string(),
                    ok: false,
                    elapsed_ms: 1_337,
                },
            ],
        }
    }

    #[test]
    fn round_trip_equality() {
        let snap = sample();
        let bytes = encode(&snap).expect("encode should succeed");
        let decoded = decode(&bytes).expect("decode should succeed");
        assert_eq!(snap, decoded);
    }

    #[test]
    fn round_trip_default() {
        let snap = EngineSnapshot::default();
        assert_eq!(
            snap.schema_version,
            EngineSnapshot::current_schema_version()
        );
        let bytes = encode(&snap).expect("encode should succeed");
        let decoded = decode(&bytes).expect("decode should succeed");
        assert_eq!(snap, decoded);
    }

    #[test]
    fn encoding_is_deterministic() {
        let snap = sample();
        let a = encode(&snap).expect("encode should succeed");
        let b = encode(&snap).expect("encode should succeed");
        assert_eq!(a, b, "same input must produce identical bytes");
    }

    #[test]
    fn decoding_garbage_returns_err() {
        // A buffer of arbitrary bytes is overwhelmingly unlikely to be a valid
        // snapshot; decoding must surface an error rather than panic.
        let garbage = vec![0xFFu8; 7];
        let result = decode(&garbage);
        assert!(result.is_err(), "garbage bytes must decode to Err");
    }

    #[test]
    fn bitcode_is_smaller_than_json() {
        let snap = sample();
        let bitcode_bytes = encode(&snap).expect("encode should succeed");
        let json_bytes = serde_json::to_vec(&snap).expect("json encode should succeed");
        assert!(
            bitcode_bytes.len() < json_bytes.len(),
            "bitcode ({} bytes) should be strictly smaller than json ({} bytes)",
            bitcode_bytes.len(),
            json_bytes.len(),
        );
    }
}
