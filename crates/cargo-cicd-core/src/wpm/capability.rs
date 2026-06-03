//! wpm capability model.

use std::path::PathBuf;

/// Cached result of a wpm binary capability scan.
pub struct WpmCapabilityCache {
    /// True when wpm/wasm4pm binary is confirmed available.
    pub is_available: bool,
    /// Path to the wpm binary if found.
    pub binary_path: Option<PathBuf>,
}

impl WpmCapabilityCache {
    /// Detect wpm availability by checking WPM_BIN env var and PATH.
    pub fn detect() -> Self {
        // Check WPM_BIN environment variable first.
        if let Ok(bin) = std::env::var("WPM_BIN") {
            let p = PathBuf::from(&bin);
            if p.is_file() {
                return Self {
                    is_available: true,
                    binary_path: Some(p),
                };
            }
        }

        // Check for `wpm` on PATH.
        let found = which_on_path("wpm").or_else(|| which_on_path("wasm4pm"));
        Self {
            is_available: found.is_some(),
            binary_path: found,
        }
    }
}

fn which_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
