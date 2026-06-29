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

