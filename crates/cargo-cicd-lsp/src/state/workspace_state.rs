//! WorkspaceState — workspace root tracking and snapshot refresh.

use std::collections::HashMap;
use std::path::PathBuf;

/// Tracks the current workspace root path and open document texts.
#[derive(Debug, Default)]
pub struct WorkspaceState {
    pub root: Option<PathBuf>,
    /// Text content of open documents, keyed by URI string.
    pub document_texts: HashMap<String, String>,
}

impl WorkspaceState {
    /// Create a new, empty WorkspaceState.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the workspace root.
    pub fn set_root(&mut self, path: PathBuf) {
        self.root = Some(path);
    }

    /// Return the current root path, if set.
    pub fn root(&self) -> Option<&PathBuf> {
        self.root.as_ref()
    }

    /// Clear the workspace root.
    pub fn clear(&mut self) {
        self.root = None;
    }

    /// Store the text content for a document URI.
    pub fn set_document_text(&mut self, uri: impl Into<String>, text: impl Into<String>) {
        self.document_texts.insert(uri.into(), text.into());
    }

    /// Retrieve the text content for a document URI.
    pub fn get_document_text(&self, uri: &str) -> Option<&str> {
        self.document_texts.get(uri).map(|s| s.as_str())
    }

    /// Remove the text content for a document URI.
    pub fn remove_document_text(&mut self, uri: &str) {
        self.document_texts.remove(uri);
    }
}
