use std::path::PathBuf;
use std::time::{Instant, SystemTime};

use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// ConfigWatcher
// ---------------------------------------------------------------------------

/// Polls a config file for modification by comparing filesystem `mtime`.
///
/// This is an intentionally simple, dependency-free implementation.  It does
/// not use `inotify`, `kqueue`, or the `notify` crate — only `std::fs::metadata`.
/// For applications that need low-latency hot-reload, replace the polling loop
/// with a background thread + channel or the `notify` crate.
///
/// ## Usage
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use project_config::watcher::ConfigWatcher;
///
/// let mut watcher = ConfigWatcher::new(PathBuf::from("project.toml"));
///
/// loop {
///     if watcher.has_changed() {
///         println!("Config changed — reloading…");
///         // re-invoke load_config() here
///     }
///     std::thread::sleep(std::time::Duration::from_secs(5));
/// }
/// ```
#[derive(Debug)]
pub struct ConfigWatcher {
    /// Path to the file being watched.
    path: PathBuf,
    /// When the watcher was constructed or last confirmed the file state.
    last_check: Instant,
    /// The `mtime` observed at the previous `has_changed()` call (or at
    /// construction time).  `None` when the file did not exist yet.
    last_mtime: Option<SystemTime>,
}

impl ConfigWatcher {
    /// Create a new watcher for `path`.
    ///
    /// Records the current `mtime` (or `None` if the file does not yet exist)
    /// as the baseline.  The first call to [`has_changed`] will return `false`
    /// unless the file has been modified between `new()` and that call.
    pub fn new(path: PathBuf) -> Self {
        let last_mtime = read_mtime(&path);
        debug!(path = %path.display(), mtime = ?last_mtime, "ConfigWatcher initialised");
        Self {
            path,
            last_check: Instant::now(),
            last_mtime,
        }
    }

    /// Check whether the file has been modified since the last call.
    ///
    /// Returns `true` once per modification event: after returning `true` the
    /// internal baseline is updated so the next call returns `false` unless
    /// the file changes again.
    ///
    /// Special cases:
    /// - File created after the watcher was constructed → returns `true`.
    /// - File deleted after being present → returns `true`.
    /// - `mtime` unavailable (permissions, unsupported OS) → returns `false`
    ///   and logs a warning.
    pub fn has_changed(&mut self) -> bool {
        let current_mtime = read_mtime(&self.path);
        let changed = current_mtime != self.last_mtime;
        if changed {
            debug!(
                path = %self.path.display(),
                old_mtime = ?self.last_mtime,
                new_mtime = ?current_mtime,
                "config file change detected"
            );
            self.last_mtime = current_mtime;
        }
        self.last_check = Instant::now();
        changed
    }

    /// Return the path being watched.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Return the instant when `has_changed()` was last called (or when the
    /// watcher was constructed if `has_changed()` has never been called).
    pub fn last_check(&self) -> Instant {
        self.last_check
    }

    /// Return the most recently observed `mtime`, or `None` if the file does
    /// not exist / the mtime could not be read.
    pub fn last_mtime(&self) -> Option<SystemTime> {
        self.last_mtime
    }
}

// ---------------------------------------------------------------------------
// Internal helper
// ---------------------------------------------------------------------------

fn read_mtime(path: &PathBuf) -> Option<SystemTime> {
    match std::fs::metadata(path) {
        Ok(meta) => match meta.modified() {
            Ok(t) => Some(t),
            Err(e) => {
                warn!(path = %path.display(), error = %e, "could not read mtime");
                None
            }
        },
        Err(_) => None, // file absent or inaccessible
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as IoWrite;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn new_watcher_has_changed_returns_false_immediately() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut watcher = ConfigWatcher::new(tmp.path().to_path_buf());
        // File has not changed since watcher was created.
        assert!(
            !watcher.has_changed(),
            "has_changed() must return false for an unchanged file"
        );
    }

    #[test]
    fn watcher_detects_file_modification() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        let mut watcher = ConfigWatcher::new(tmp.path().to_path_buf());

        // Ensure at least 1 second passes so mtime granularity (1 s on many
        // filesystems) is sufficient to register a change.
        thread::sleep(Duration::from_millis(1100));
        tmp.write_all(b"# modified\n").unwrap();
        tmp.flush().unwrap();

        assert!(
            watcher.has_changed(),
            "has_changed() must return true after file modification"
        );
    }

    #[test]
    fn watcher_returns_false_after_change_is_consumed() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        let mut watcher = ConfigWatcher::new(tmp.path().to_path_buf());

        thread::sleep(Duration::from_millis(1100));
        tmp.write_all(b"# changed\n").unwrap();
        tmp.flush().unwrap();

        let first = watcher.has_changed();
        let second = watcher.has_changed();
        assert!(first, "first call should detect the change");
        assert!(!second, "second call should return false (no new change)");
    }

    #[test]
    fn watcher_nonexistent_file_does_not_panic() {
        let mut watcher =
            ConfigWatcher::new(PathBuf::from("/tmp/__does_not_exist_watcher_test__.toml"));
        // Should silently return false — the file never existed.
        assert!(!watcher.has_changed());
    }

    #[test]
    fn watcher_detects_file_creation() {
        // Start watching a path where the file does not yet exist.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("late_created.toml");
        let mut watcher = ConfigWatcher::new(path.clone());
        assert!(!watcher.has_changed(), "file not created yet");

        // Now create the file.
        std::fs::write(&path, b"[project]\nname = \"new\"\n").unwrap();

        assert!(
            watcher.has_changed(),
            "file creation must be detected as a change"
        );
    }

    #[test]
    fn watcher_exposes_path() {
        let p = PathBuf::from("/tmp/sentinel.toml");
        let watcher = ConfigWatcher::new(p.clone());
        assert_eq!(watcher.path(), &p);
    }
}
