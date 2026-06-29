//! CapabilityCache — cached wpm capability state.

/// Cached result of wpm capability detection.
#[derive(Debug, Default, Clone)]
pub struct CapabilityCache {
    /// Whether the wpm binary was found on PATH.
    pub wpm_available: bool,
    /// Whether the wpm runtime court is confirmed.
    pub court_confirmed: bool,
    /// The wpm version string, if available.
    pub wpm_version: Option<String>,
    /// Whether the file watcher capability has been registered with the client.
    pub watcher_registered: bool,
}

impl CapabilityCache {
    /// Create an empty cache (nothing confirmed).
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark wpm as available with the given version string.
    pub fn set_available(&mut self, version: impl Into<String>) {
        self.wpm_available = true;
        self.wpm_version = Some(version.into());
    }

    /// Mark wpm as unavailable.
    pub fn set_unavailable(&mut self) {
        self.wpm_available = false;
        self.court_confirmed = false;
        self.wpm_version = None;
    }

    /// Mark the runtime court as confirmed.
    pub fn confirm_court(&mut self) {
        self.court_confirmed = true;
    }

    /// Clear all cached state.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
