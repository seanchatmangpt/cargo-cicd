//! DiagnosticStore — keyed storage of CicdFinding per URI.

use cargo_cicd_core::diagnostics::CicdFinding;
use std::collections::HashMap;

/// Stores diagnostic findings keyed by document URI.
pub struct DiagnosticStore {
    inner: HashMap<String, Vec<CicdFinding>>,
}

impl DiagnosticStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Insert a finding for the given URI.
    pub fn insert(&mut self, uri: String, finding: CicdFinding) {
        self.inner.entry(uri).or_default().push(finding);
    }

    /// Remove only findings whose code string matches `code_str` for the given URI.
    /// Other findings for the same URI are preserved.
    pub fn remove_code(&mut self, uri: &str, code_str: &str) {
        if let Some(findings) = self.inner.get_mut(uri) {
            findings.retain(|f| f.code.as_str() != code_str);
        }
    }

    /// Return all findings for the given URI.
    pub fn get_all(&self, uri: &str) -> Vec<&CicdFinding> {
        self.inner
            .get(uri)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Remove all findings for the given URI.
    pub fn clear_uri(&mut self, uri: &str) {
        self.inner.remove(uri);
    }
}

impl Default for DiagnosticStore {
    fn default() -> Self {
        Self::new()
    }
}
