//! Glob-based file event watchers for the workspace.

use std::path::PathBuf;

/// A file system event emitted by the watcher.
#[derive(Debug, Clone)]
pub struct WatchEvent {
    /// The path that changed.
    pub path: PathBuf,
    /// The kind of change.
    pub kind: WatchEventKind,
}

/// The kind of file system change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEventKind {
    Created,
    Modified,
    Deleted,
}

impl WatchEvent {
    /// Construct a new WatchEvent.
    pub fn new(path: PathBuf, kind: WatchEventKind) -> Self {
        Self { path, kind }
    }
}
