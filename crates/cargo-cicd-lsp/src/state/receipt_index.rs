//! ReceiptIndex — index of receipt files present in the workspace.

use std::collections::HashSet;
use std::path::PathBuf;

/// Tracks the set of receipt file paths known to the workspace.
#[derive(Debug, Default)]
pub struct ReceiptIndex {
    paths: HashSet<PathBuf>,
}

impl ReceiptIndex {
    /// Create a new, empty ReceiptIndex.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a receipt path.
    pub fn insert(&mut self, path: PathBuf) {
        self.paths.insert(path);
    }

    /// Remove a receipt path.
    pub fn remove(&mut self, path: &PathBuf) {
        self.paths.remove(path);
    }

    /// Returns true if the given path is indexed.
    pub fn contains(&self, path: &PathBuf) -> bool {
        self.paths.contains(path)
    }

    /// Returns all indexed receipt paths.
    pub fn all(&self) -> impl Iterator<Item = &PathBuf> {
        self.paths.iter()
    }

    /// Clear all indexed paths.
    pub fn clear(&mut self) {
        self.paths.clear();
    }

    /// Number of indexed receipts.
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Returns true if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}
