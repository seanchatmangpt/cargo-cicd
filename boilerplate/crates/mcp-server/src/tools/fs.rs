//! `read_file` tool — reads a workspace file with path-traversal protection.
//!
//! # Security
//!
//! **Path traversal protection is mandatory.**  Before any I/O, this module
//! canonicalizes both the requested path and the workspace root, then
//! asserts that the requested path starts with the workspace root.  Requests
//! that escape the workspace (e.g. `../../etc/passwd`, symlinks pointing
//! outside) are rejected with [`McpError::PermissionDenied`].
//!
//! # Size limit
//!
//! Files larger than `max_bytes` are rejected to prevent the AI assistant
//! from accidentally ingesting huge binaries.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::McpError;

// ---------------------------------------------------------------------------
// Output shape
// ---------------------------------------------------------------------------

/// Payload returned by the `read_file` tool.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadFileResult {
    /// The canonicalized, workspace-relative path that was read.
    pub path: String,

    /// UTF-8 content of the file.
    pub content: String,

    /// File size in bytes.
    pub size_bytes: usize,
}

// ---------------------------------------------------------------------------
// Core logic (called from the tool handler)
// ---------------------------------------------------------------------------

/// Read the file at `requested_path` after verifying it is inside
/// `workspace_root`.
///
/// # Errors
///
/// - [`McpError::PermissionDenied`] if the canonicalized path escapes the
///   workspace root.
/// - [`McpError::WorkspaceError`] if `workspace_root` cannot be canonicalized.
/// - [`McpError::IoError`] if the file cannot be read.
/// - [`McpError::WorkspaceError`] if the file is larger than `max_bytes` or
///   contains non-UTF-8 bytes.
pub fn read_file_safe(
    workspace_root: &Path,
    requested_path: &str,
    max_bytes: usize,
) -> Result<ReadFileResult, McpError> {
    // 1. Canonicalize the workspace root so symlinks in the root itself don't
    //    bypass the check.
    let canonical_root = canonicalize_workspace_root(workspace_root)?;

    // 2. Resolve the requested path relative to the workspace root.
    //    We intentionally prepend workspace_root so that absolute paths from
    //    the caller are re-anchored inside the workspace.
    let joined = resolve_path(&canonical_root, requested_path);

    // 3. Canonicalize the joined path (resolves `..`, symlinks, etc.).
    //    We use our own helper so we can give a clear error if the path does
    //    not exist.
    let canonical_file = std::fs::canonicalize(&joined).map_err(|e| {
        McpError::IoError(std::io::Error::new(
            e.kind(),
            format!("cannot resolve '{}': {e}", joined.display()),
        ))
    })?;

    // 4. PATH TRAVERSAL CHECK — the critical security gate.
    if !canonical_file.starts_with(&canonical_root) {
        return Err(McpError::PermissionDenied(format!(
            "'{}' resolves to '{}' which is outside the workspace root '{}'",
            requested_path,
            canonical_file.display(),
            canonical_root.display(),
        )));
    }

    // 5. Size check before reading the whole file.
    let metadata = std::fs::metadata(&canonical_file)?;
    let file_len = metadata.len() as usize;
    if file_len > max_bytes {
        return Err(McpError::WorkspaceError(anyhow::anyhow!(
            "file '{}' is {file_len} bytes, which exceeds the {max_bytes}-byte limit",
            canonical_file.display(),
        )));
    }

    // 6. Read the file.
    let raw = std::fs::read(&canonical_file)?;

    // 7. Validate UTF-8.
    let content = String::from_utf8(raw).map_err(|_| {
        McpError::WorkspaceError(anyhow::anyhow!(
            "file '{}' is not valid UTF-8; binary files are not supported",
            canonical_file.display(),
        ))
    })?;

    // 8. Compute relative path for the response (nicer than the full canonical
    //    path for the caller).
    let relative = canonical_file
        .strip_prefix(&canonical_root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| canonical_file.to_string_lossy().into_owned());

    Ok(ReadFileResult {
        path: relative,
        size_bytes: content.len(),
        content,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Canonicalize the workspace root, mapping the error to [`McpError`].
fn canonicalize_workspace_root(workspace_root: &Path) -> Result<PathBuf, McpError> {
    std::fs::canonicalize(workspace_root).map_err(|e| {
        McpError::WorkspaceError(anyhow::anyhow!(
            "cannot canonicalize workspace root '{}': {e}",
            workspace_root.display()
        ))
    })
}

/// Construct the absolute path we will read.
///
/// Rules:
/// - If `requested` is an absolute path, re-anchor it under
///   `canonical_root` by stripping the leading `/`.
/// - If `requested` is a relative path, join it to `canonical_root`.
///
/// This ensures the path is always inside the workspace even before
/// canonicalization.
fn resolve_path(canonical_root: &Path, requested: &str) -> PathBuf {
    let p = Path::new(requested);
    if p.is_absolute() {
        // Strip the leading separator and join to workspace root.
        // e.g. "/etc/passwd" → workspace_root/etc/passwd
        let stripped = p
            .components()
            .filter(|c| !matches!(c, std::path::Component::RootDir | std::path::Component::Prefix(_)))
            .collect::<PathBuf>();
        canonical_root.join(stripped)
    } else {
        canonical_root.join(p)
    }
}

/// Serialize a [`ReadFileResult`] to JSON.
pub fn file_result_to_json(result: &ReadFileResult) -> Result<Value, McpError> {
    serde_json::to_value(result).map_err(McpError::SerializationError)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_workspace() -> TempDir {
        TempDir::new().expect("tempdir")
    }

    #[test]
    fn reads_file_inside_workspace() {
        let dir = make_workspace();
        let file = dir.path().join("hello.txt");
        fs::write(&file, "hello world").unwrap();

        let result = read_file_safe(dir.path(), "hello.txt", 1024 * 1024).unwrap();
        assert_eq!(result.content, "hello world");
        assert_eq!(result.size_bytes, 11);
        assert_eq!(result.path, "hello.txt");
    }

    #[test]
    fn path_traversal_via_dotdot_is_rejected() {
        let dir = make_workspace();
        // Create a file outside the workspace in /tmp.
        let outside = TempDir::new().unwrap();
        let secret = outside.path().join("secret.txt");
        fs::write(&secret, "top secret").unwrap();

        // Try to escape via ../secret.txt-style path.
        let escaped = format!("../../{}", secret.display());
        let result = read_file_safe(dir.path(), &escaped, 1024 * 1024);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        // Should be either PermissionDenied or IoError (path resolves to
        // non-existent when re-anchored).
        assert!(
            msg.contains("outside the workspace root") || msg.contains("cannot resolve"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn absolute_path_is_reanchored_inside_workspace() {
        let dir = make_workspace();
        let sub = dir.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        let file = sub.join("data.txt");
        fs::write(&file, "data").unwrap();

        // Request with an absolute path — should be reanchored inside workspace.
        let result = read_file_safe(dir.path(), "/subdir/data.txt", 1024 * 1024).unwrap();
        assert_eq!(result.content, "data");
    }

    #[test]
    fn file_exceeding_size_limit_is_rejected() {
        let dir = make_workspace();
        let file = dir.path().join("big.txt");
        // Write 10 bytes, limit to 5 bytes.
        fs::write(&file, "0123456789").unwrap();

        let result = read_file_safe(dir.path(), "big.txt", 5);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("exceeds"), "unexpected error: {msg}");
    }

    #[test]
    fn missing_file_returns_io_error() {
        let dir = make_workspace();
        let result = read_file_safe(dir.path(), "nonexistent.txt", 1024 * 1024);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_path_relative() {
        let root = PathBuf::from("/workspace");
        let resolved = resolve_path(&root, "src/main.rs");
        assert_eq!(resolved, PathBuf::from("/workspace/src/main.rs"));
    }

    #[test]
    fn resolve_path_absolute_reanchors() {
        let root = PathBuf::from("/workspace");
        let resolved = resolve_path(&root, "/etc/passwd");
        assert_eq!(resolved, PathBuf::from("/workspace/etc/passwd"));
    }
}
