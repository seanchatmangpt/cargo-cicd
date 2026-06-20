//! BLAKE3 content-addressed artifact hashing and Merkle chain construction.
//!
//! All public functions return `Result<T>` and never panic (silence contract).
//!
//! # Example
//!
//! ```rust,ignore
//! use std::path::Path;
//! use my_crate::advanced::fingerprint::{
//!     fingerprint_file, fingerprint_bytes, verify_file, MerkleChain,
//! };
//!
//! // Hash a file
//! let manifest = fingerprint_file(Path::new("Cargo.lock"))?;
//! println!("{} -> {}", manifest.path.display(), manifest.hash);
//!
//! // Hash raw bytes
//! let hash = fingerprint_bytes(b"hello world");
//! println!("hash: {}", hash);
//!
//! // Verify integrity
//! let ok = verify_file(Path::new("Cargo.lock"), &manifest.hash)?;
//! assert!(ok);
//!
//! // Build a Merkle chain over multiple content hashes
//! let mut chain = MerkleChain::new();
//! chain.push(hash);
//! let root = chain.root();
//! ```

use anyhow::{Context, Result};
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::SystemTime;

// ─── ContentHash ─────────────────────────────────────────────────────────────

/// A BLAKE3 content hash — a newtype wrapping [`blake3::Hash`].
///
/// Displays as a 64-character lowercase hex string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentHash(blake3::Hash);

impl ContentHash {
    /// Returns the raw 32-byte array.
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    /// Returns the hash as a lowercase hex string (64 characters).
    pub fn to_hex(&self) -> String {
        self.0.to_hex().to_string()
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}

impl FromStr for ContentHash {
    type Err = anyhow::Error;

    /// Parse a 64-character lowercase hex string into a [`ContentHash`].
    fn from_str(s: &str) -> Result<Self> {
        let hash = blake3::Hash::from_hex(s)
            .map_err(|e| anyhow::anyhow!("invalid BLAKE3 hex string: {e}"))?;
        Ok(ContentHash(hash))
    }
}

// ─── ArtifactManifest ────────────────────────────────────────────────────────

/// Metadata and content hash for a single artifact file.
#[derive(Debug, Clone)]
pub struct ArtifactManifest {
    /// Canonical path to the artifact.
    pub path: PathBuf,
    /// BLAKE3 hash of the file's contents.
    pub hash: ContentHash,
    /// File size in bytes at the time of hashing.
    pub size_bytes: u64,
    /// Last-modified timestamp at the time of hashing.
    pub modified: SystemTime,
}

// ─── Public functions ─────────────────────────────────────────────────────────

/// Compute a [`ContentHash`] from a byte slice without touching the filesystem.
///
/// This is allocation-free for the hash computation itself.
pub fn fingerprint_bytes(data: &[u8]) -> ContentHash {
    ContentHash(blake3::hash(data))
}

/// Hash the contents of `path` and return a full [`ArtifactManifest`].
///
/// Reads the entire file into memory. For very large files (> a few GB) use a
/// streaming approach via [`blake3::Hasher`] directly.
///
/// # Errors
///
/// Returns an error if the file cannot be read or its metadata cannot be
/// queried.
pub fn fingerprint_file(path: &Path) -> Result<ArtifactManifest> {
    let data = fs::read(path)
        .with_context(|| format!("failed to read file for fingerprinting: {}", path.display()))?;

    let meta = fs::metadata(path)
        .with_context(|| format!("failed to stat file: {}", path.display()))?;

    Ok(ArtifactManifest {
        path: path.to_path_buf(),
        hash: fingerprint_bytes(&data),
        size_bytes: meta.len(),
        modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
    })
}

/// Re-hash `path` and compare with `expected`.
///
/// Returns `Ok(true)` if the file still matches `expected`, `Ok(false)` if
/// the content has changed.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub fn verify_file(path: &Path, expected: &ContentHash) -> Result<bool> {
    let manifest = fingerprint_file(path)?;
    Ok(&manifest.hash == expected)
}

/// Hash `path` in streaming chunks, keeping memory usage bounded to
/// `chunk_size` bytes at a time.  Use this for large files where loading the
/// whole contents at once would be prohibitive.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read.
pub fn fingerprint_file_streaming(path: &Path, chunk_size: usize) -> Result<ContentHash> {
    let mut file = fs::File::open(path)
        .with_context(|| format!("failed to open file: {}", path.display()))?;

    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; chunk_size];
    loop {
        let n = file
            .read(&mut buf)
            .with_context(|| format!("failed to read chunk from: {}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(ContentHash(hasher.finalize()))
}

// ─── MerkleChain ─────────────────────────────────────────────────────────────

/// An append-only chain of [`ContentHash`] values whose `root` is the BLAKE3
/// hash of all constituent hashes concatenated in order.
///
/// This is not a full Merkle tree (no sibling pairing), but provides a
/// deterministic, order-sensitive commitment over a sequence of artifacts —
/// useful for pipeline-stage manifests.
#[derive(Debug, Clone, Default)]
pub struct MerkleChain {
    entries: Vec<ContentHash>,
}

impl MerkleChain {
    /// Create an empty chain.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a [`ContentHash`] to the chain.
    pub fn push(&mut self, hash: ContentHash) {
        self.entries.push(hash);
    }

    /// Return the number of entries in the chain.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return `true` if the chain has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Compute the chain root: `BLAKE3(hash[0] || hash[1] || … || hash[n-1])`.
    ///
    /// Returns the hash of an empty byte string if the chain is empty
    /// (deterministic, stable sentinel value).
    pub fn root(&self) -> ContentHash {
        let mut hasher = blake3::Hasher::new();
        for entry in &self.entries {
            hasher.update(entry.as_bytes());
        }
        ContentHash(hasher.finalize())
    }

    /// Return an iterator over the entries in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &ContentHash> {
        self.entries.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ── fingerprint_bytes ───────────────────────────────────────────────────

    #[test]
    fn fingerprint_bytes_is_deterministic() {
        let h1 = fingerprint_bytes(b"hello world");
        let h2 = fingerprint_bytes(b"hello world");
        assert_eq!(h1, h2);
    }

    #[test]
    fn fingerprint_bytes_different_data_different_hash() {
        let h1 = fingerprint_bytes(b"foo");
        let h2 = fingerprint_bytes(b"bar");
        assert_ne!(h1, h2);
    }

    #[test]
    fn fingerprint_empty_bytes_is_stable() {
        let h = fingerprint_bytes(b"");
        // Must not panic and must produce a 64-char hex string.
        assert_eq!(h.to_hex().len(), 64);
    }

    // ── ContentHash Display / FromStr ───────────────────────────────────────

    #[test]
    fn content_hash_round_trips_via_display_and_from_str() {
        let original = fingerprint_bytes(b"round-trip test");
        let hex = original.to_string();
        assert_eq!(hex.len(), 64);

        let parsed: ContentHash = hex.parse().expect("parse should succeed");
        assert_eq!(original, parsed);
    }

    #[test]
    fn content_hash_from_str_rejects_invalid_hex() {
        let result = "not-valid-hex-at-all".parse::<ContentHash>();
        assert!(result.is_err());
    }

    // ── fingerprint_file ────────────────────────────────────────────────────

    #[test]
    fn fingerprint_file_returns_correct_hash() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.bin");
        fs::write(&path, b"content").unwrap();

        let manifest = fingerprint_file(&path).unwrap();

        assert_eq!(manifest.hash, fingerprint_bytes(b"content"));
        assert_eq!(manifest.size_bytes, 7);
        assert_eq!(manifest.path, path);
    }

    #[test]
    fn fingerprint_file_nonexistent_returns_error() {
        let result = fingerprint_file(Path::new("/tmp/__nonexistent_fp_test__.bin"));
        assert!(result.is_err());
    }

    #[test]
    fn fingerprint_file_matches_streaming_result() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("stream.bin");
        let payload = vec![0xABu8; 8192]; // 8 KB
        fs::write(&path, &payload).unwrap();

        let manifest = fingerprint_file(&path).unwrap();
        let streaming = fingerprint_file_streaming(&path, 1024).unwrap();

        assert_eq!(
            manifest.hash, streaming,
            "batch and streaming hashes must agree"
        );
    }

    // ── verify_file ─────────────────────────────────────────────────────────

    #[test]
    fn verify_file_returns_true_for_unchanged_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("check.txt");
        fs::write(&path, b"unchanged").unwrap();

        let manifest = fingerprint_file(&path).unwrap();
        assert!(verify_file(&path, &manifest.hash).unwrap());
    }

    #[test]
    fn verify_file_returns_false_after_modification() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("mutate.txt");
        fs::write(&path, b"original").unwrap();

        let manifest = fingerprint_file(&path).unwrap();

        // Mutate the file.
        fs::write(&path, b"modified").unwrap();

        assert!(!verify_file(&path, &manifest.hash).unwrap());
    }

    // ── MerkleChain ─────────────────────────────────────────────────────────

    #[test]
    fn empty_chain_root_is_stable() {
        let chain = MerkleChain::new();
        let r1 = chain.root();
        let r2 = chain.root();
        assert_eq!(r1, r2);
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn chain_root_changes_when_entry_added() {
        let mut chain = MerkleChain::new();
        let root_before = chain.root();

        chain.push(fingerprint_bytes(b"entry-one"));
        let root_after = chain.root();

        assert_ne!(root_before, root_after);
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn chain_root_is_order_sensitive() {
        let h1 = fingerprint_bytes(b"alpha");
        let h2 = fingerprint_bytes(b"beta");

        let mut chain_ab = MerkleChain::new();
        chain_ab.push(h1.clone());
        chain_ab.push(h2.clone());

        let mut chain_ba = MerkleChain::new();
        chain_ba.push(h2);
        chain_ba.push(h1);

        assert_ne!(
            chain_ab.root(),
            chain_ba.root(),
            "different insertion orders must yield different roots"
        );
    }

    #[test]
    fn chain_root_is_deterministic_across_identical_chains() {
        let build_chain = || {
            let mut c = MerkleChain::new();
            c.push(fingerprint_bytes(b"a"));
            c.push(fingerprint_bytes(b"b"));
            c.push(fingerprint_bytes(b"c"));
            c
        };

        assert_eq!(build_chain().root(), build_chain().root());
    }

    #[test]
    fn chain_iter_yields_entries_in_order() {
        let hashes: Vec<ContentHash> = (0u8..5)
            .map(|i| fingerprint_bytes(&[i]))
            .collect();

        let mut chain = MerkleChain::new();
        for h in &hashes {
            chain.push(h.clone());
        }

        let collected: Vec<&ContentHash> = chain.iter().collect();
        assert_eq!(collected.len(), hashes.len());
        for (a, b) in collected.iter().zip(hashes.iter()) {
            assert_eq!(*a, b);
        }
    }
}
