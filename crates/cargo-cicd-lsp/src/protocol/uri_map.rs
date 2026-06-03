//! URI to path helpers.

use std::path::PathBuf;
use tower_lsp::lsp_types::Url;

/// Convert a `Url` to a `PathBuf`, returning `None` if not a file URI.
pub fn uri_to_path(uri: &Url) -> Option<PathBuf> {
    uri.to_file_path().ok()
}

/// Convert a `PathBuf` to a `Url`, returning `None` on failure.
pub fn path_to_uri(path: &PathBuf) -> Option<Url> {
    Url::from_file_path(path).ok()
}

/// Convert a path string to a `Url`, returning `None` on failure.
pub fn path_str_to_uri(path: &str) -> Option<Url> {
    Url::from_file_path(path).ok()
}
