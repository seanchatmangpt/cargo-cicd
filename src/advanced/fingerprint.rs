//! Content-addressed BLAKE3 fingerprinting and Merkle workspace digests.
//!
//! This module exposes a small, engine-shaped API over the `blake3` crate:
//!
//! * [`Fingerprint`] — a 32-byte content hash newtype with hex/`Display` output.
//! * [`hash_bytes`] — hash an in-memory byte slice.
//! * [`hash_file`] — stream a file through the hasher without buffering it whole.
//! * [`workspace_digest`] — fold a set of `(path, child-hash)` entries into a
//!   single deterministic root that is order-independent of the input order but
//!   sensitive to any path or content change.

use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Size in bytes of a BLAKE3 digest.
pub const FINGERPRINT_LEN: usize = 32;

/// Read buffer size used while streaming a file through the hasher (64 KiB).
const READ_BUFFER_LEN: usize = 64 * 1024;

/// Domain-separation context for workspace Merkle roots. Using a distinct
/// derive-key context keeps workspace roots from colliding with raw content
/// hashes even when the underlying bytes happen to coincide.
const WORKSPACE_CONTEXT: &str = "cargo-cicd advanced::fingerprint workspace-digest v1";

/// A content-addressed BLAKE3 fingerprint (32 raw bytes).
///
/// `Fingerprint` is a thin newtype over the digest. Two fingerprints are equal
/// exactly when their underlying bytes match, and the [`Display`](fmt::Display)
/// / [`to_hex`](Fingerprint::to_hex) representations are lowercase hexadecimal.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Fingerprint([u8; FINGERPRINT_LEN]);

impl Fingerprint {
    /// Construct a fingerprint from raw digest bytes.
    pub const fn from_bytes(bytes: [u8; FINGERPRINT_LEN]) -> Self {
        Fingerprint(bytes)
    }

    /// Borrow the underlying 32 digest bytes.
    pub fn as_bytes(&self) -> &[u8; FINGERPRINT_LEN] {
        &self.0
    }

    /// Render the digest as a lowercase hexadecimal string.
    pub fn to_hex(&self) -> String {
        let mut out = String::with_capacity(FINGERPRINT_LEN * 2);
        use fmt::Write as _;
        for byte in &self.0 {
            // `{:02x}` writes exactly two lowercase hex digits per byte.
            let _ = write!(out, "{byte:02x}");
        }
        out
    }
}

impl From<blake3::Hash> for Fingerprint {
    fn from(hash: blake3::Hash) -> Self {
        Fingerprint(*hash.as_bytes())
    }
}

impl From<[u8; FINGERPRINT_LEN]> for Fingerprint {
    fn from(bytes: [u8; FINGERPRINT_LEN]) -> Self {
        Fingerprint(bytes)
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fingerprint({})", self.to_hex())
    }
}

/// Hash an in-memory byte slice into a [`Fingerprint`].
pub fn hash_bytes(data: &[u8]) -> Fingerprint {
    blake3::hash(data).into()
}

/// Stream the contents of `path` through a BLAKE3 hasher.
///
/// The file is read in fixed-size chunks so arbitrarily large files never need
/// to be buffered into memory at once. The resulting fingerprint is identical
/// to [`hash_bytes`] of the same content.
pub fn hash_file(path: &Path) -> io::Result<Fingerprint> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; READ_BUFFER_LEN];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hasher.finalize().into())
}

/// Compute a deterministic Merkle-style root over a set of workspace entries.
///
/// Each entry pairs a path with the fingerprint of that path's content. The
/// entries are sorted by path before folding, so the resulting root is
/// independent of the order in which entries are supplied, yet sensitive to any
/// change in a path or its child hash. A leaf is bound to its path by hashing
/// the length-prefixed path bytes alongside the child digest, which prevents
/// ambiguity between adjacent fields.
pub fn workspace_digest(entries: &[(PathBuf, Fingerprint)]) -> Fingerprint {
    // Work on a sorted, owned view so the caller's slice is never mutated and
    // the fold order is fully deterministic.
    let mut sorted: Vec<&(PathBuf, Fingerprint)> = entries.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    // Domain-separated derived-key hasher keeps workspace roots distinct from
    // raw content hashes.
    let mut hasher = blake3::Hasher::new_derive_key(WORKSPACE_CONTEXT);

    // Bind in the entry count so digests of differing cardinality cannot
    // collide via empty/padding leaves.
    hasher.update(&(sorted.len() as u64).to_le_bytes());

    for (path, child) in sorted {
        let path_str = path.to_string_lossy();
        let path_bytes = path_str.as_bytes();
        // Length-prefix the path so leaf fields are unambiguously framed.
        hasher.update(&(path_bytes.len() as u64).to_le_bytes());
        hasher.update(path_bytes);
        hasher.update(child.as_bytes());
    }

    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    #[test]
    fn hash_bytes_is_deterministic() {
        let a = hash_bytes(b"cargo-cicd process data");
        let b = hash_bytes(b"cargo-cicd process data");
        assert_eq!(a, b, "identical input must produce identical fingerprints");
        assert_eq!(a.to_hex(), b.to_hex());
        assert_eq!(a.to_hex().len(), FINGERPRINT_LEN * 2);
        // Display and to_hex agree.
        assert_eq!(format!("{a}"), a.to_hex());
        // as_bytes round-trips through from_bytes.
        assert_eq!(Fingerprint::from_bytes(*a.as_bytes()), a);
    }

    #[test]
    fn hash_bytes_is_sensitive_to_single_byte() {
        let original = hash_bytes(b"workspace-state-0");
        let mutated = hash_bytes(b"workspace-state-1");
        assert_ne!(
            original, mutated,
            "a single-byte change must alter the fingerprint"
        );
    }

    #[test]
    fn hash_file_matches_hash_bytes_via_streaming() {
        // Span several read buffers plus a partial tail to exercise the loop.
        let content = vec![0xABu8; READ_BUFFER_LEN * 3 + 17];
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        file.write_all(&content).expect("write content");
        file.flush().expect("flush content");

        let streamed = hash_file(file.path()).expect("hash file");
        let in_memory = hash_bytes(&content);
        assert_eq!(
            streamed, in_memory,
            "streaming hash_file must match hash_bytes for identical content"
        );
    }

    #[test]
    fn hash_file_errors_on_missing_path() {
        let result = hash_file(Path::new("/this/path/should/not/exist/xyz123"));
        assert!(result.is_err(), "missing file must surface an io error");
    }

    #[test]
    fn workspace_digest_is_order_independent() {
        let fp = |s: &[u8]| hash_bytes(s);
        let forward = vec![
            (PathBuf::from("src/lib.rs"), fp(b"lib")),
            (PathBuf::from("src/main.rs"), fp(b"main")),
            (PathBuf::from("Cargo.toml"), fp(b"manifest")),
        ];
        let mut shuffled = forward.clone();
        shuffled.reverse();

        assert_eq!(
            workspace_digest(&forward),
            workspace_digest(&shuffled),
            "root must not depend on input ordering"
        );
    }

    #[test]
    fn workspace_digest_is_change_sensitive() {
        let fp = |s: &[u8]| hash_bytes(s);
        let base = vec![
            (PathBuf::from("a.rs"), fp(b"a")),
            (PathBuf::from("b.rs"), fp(b"b")),
        ];

        // Change a child content hash.
        let changed_content = vec![
            (PathBuf::from("a.rs"), fp(b"a-modified")),
            (PathBuf::from("b.rs"), fp(b"b")),
        ];
        assert_ne!(
            workspace_digest(&base),
            workspace_digest(&changed_content),
            "changing a child hash must change the root"
        );

        // Change a path.
        let changed_path = vec![
            (PathBuf::from("a.rs"), fp(b"a")),
            (PathBuf::from("c.rs"), fp(b"b")),
        ];
        assert_ne!(
            workspace_digest(&base),
            workspace_digest(&changed_path),
            "changing a path must change the root"
        );

        // Differing cardinality must not collide.
        let extra = vec![
            (PathBuf::from("a.rs"), fp(b"a")),
            (PathBuf::from("b.rs"), fp(b"b")),
            (PathBuf::from("c.rs"), fp(b"c")),
        ];
        assert_ne!(workspace_digest(&base), workspace_digest(&extra));
    }

    #[test]
    fn empty_workspace_digest_is_stable() {
        let empty: Vec<(PathBuf, Fingerprint)> = Vec::new();
        assert_eq!(workspace_digest(&empty), workspace_digest(&empty));
    }
}
