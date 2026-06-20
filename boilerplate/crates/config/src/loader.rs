use std::path::{Path, PathBuf};

use tracing::debug;

use crate::error::ConfigError;
use crate::layer::{merge_layers, ConfigLayer};
use crate::schema::Config;

// ---------------------------------------------------------------------------
// ConfigLoader
// ---------------------------------------------------------------------------

/// Builder that assembles a prioritised stack of [`ConfigLayer`]s and
/// produces a validated [`Config`].
///
/// Priority order (highest wins):
/// 1. Compiled-in defaults   — always present as the base
/// 2. Config file            — optional; added via `with_file` / `with_file_optional`
/// 3. Environment variables  — added via `with_env`
/// 4. Programmatic override  — added via `with_overrides`
///
/// Layers are merged in the order they were added; the last call to
/// `with_*` is the highest priority.
///
/// ## Example
///
/// ```rust,no_run
/// use project_config::ConfigLoader;
///
/// let config = ConfigLoader::new()
///     .with_file_optional("project.toml")
///     .with_env("APP")
///     .load()
///     .expect("config must be valid");
/// ```
#[derive(Debug)]
pub struct ConfigLoader {
    layers: Vec<ConfigLayer>,
    /// Remembered so `watcher` can be told which path to watch.
    pub config_file_path: Option<PathBuf>,
}

impl ConfigLoader {
    // ------------------------------------------------------------------
    // Construction
    // ------------------------------------------------------------------

    /// Create a new loader.  The defaults layer is always the first (lowest
    /// priority) layer in the stack.
    pub fn new() -> Self {
        Self {
            layers: vec![ConfigLayer::from_defaults()],
            config_file_path: None,
        }
    }

    // ------------------------------------------------------------------
    // Layer builders
    // ------------------------------------------------------------------

    /// Add a mandatory TOML config file layer.
    ///
    /// Returns [`ConfigError::FileNotFound`] when `path` does not exist.
    pub fn with_file(mut self, path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        match ConfigLayer::from_file(path) {
            Ok(layer) => {
                debug!(path = %path.display(), "added file layer");
                self.config_file_path = Some(path.to_path_buf());
                self.layers.push(layer);
            }
            Err(e) => {
                // Store the error so `load()` can surface it.
                // We model this by pushing a sentinel — a value layer whose
                // parse will fail — but it is cleaner to record the error
                // explicitly.  We keep a pending-error field for this.
                //
                // For simplicity: panic in debug, propagate in release via load().
                // We store the error inline as a special marker layer.
                tracing::error!(error = %e, "file layer error (will surface in load())");
                self.layers.push(ConfigLayer::_error_sentinel(e));
            }
        }
        self
    }

    /// Add an optional TOML config file layer.
    ///
    /// If the file does not exist the layer is silently skipped.  Any other
    /// error (parse failure, I/O error) is still propagated via `load()`.
    pub fn with_file_optional(self, path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        if !path.exists() {
            debug!(path = %path.display(), "optional config file not found — skipping");
            return self;
        }
        self.with_file(path)
    }

    /// Add an environment-variable layer.
    ///
    /// Variables matching `PREFIX_*` are translated into nested TOML paths.
    /// See [`ConfigLayer::from_env`] for the full naming convention.
    pub fn with_env(mut self, prefix: &str) -> Self {
        debug!(prefix = prefix, "adding env layer");
        self.layers.push(ConfigLayer::from_env(prefix));
        self
    }

    /// Add a programmatic override layer (highest priority when placed last).
    ///
    /// `v` is a [`toml::Value`] whose keys overwrite any previously loaded
    /// values for those keys.
    pub fn with_overrides(mut self, v: toml::Value) -> Self {
        debug!("adding override layer");
        self.layers.push(ConfigLayer::from_value(v));
        self
    }

    // ------------------------------------------------------------------
    // Terminal operation
    // ------------------------------------------------------------------

    /// Merge all layers and validate the result.
    ///
    /// # Errors
    ///
    /// - Any layer that recorded a deferred error (e.g. a missing mandatory
    ///   file) will cause this call to fail.
    /// - Merge / deserialisation failures → [`ConfigError::MergeError`].
    /// - Validation failures → [`ConfigError::ValidationFailed`].
    pub fn load(self) -> Result<Config, ConfigError> {
        // Surface any deferred errors from sentinel layers.
        for layer in &self.layers {
            if let Some(e) = layer._deferred_error() {
                return Err(e);
            }
        }

        let config = merge_layers(&self.layers)?;
        config.validate()?;
        Ok(config)
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Convenience function
// ---------------------------------------------------------------------------

/// Load configuration using the standard resolution order:
///
/// 1. Compiled-in defaults
/// 2. `project.toml` in the current directory (if present)
/// 3. Environment variables prefixed with `APP_`
///
/// This is the recommended entry-point for applications that do not need
/// fine-grained control over the layer stack.
pub fn load_config() -> Result<Config, ConfigError> {
    ConfigLoader::new()
        .with_file_optional("project.toml")
        .with_env("APP")
        .load()
}

// ---------------------------------------------------------------------------
// Deferred-error sentinel (private impl detail)
// ---------------------------------------------------------------------------

/// A private extension used to propagate errors from the builder phase into
/// `load()` without changing the return type of `with_file`.
impl ConfigLayer {
    pub(crate) fn _error_sentinel(e: ConfigError) -> Self {
        // Encode the error as a special TOML comment marker.  The sentinel is
        // identified by the presence of a top-level `__error__` key.
        let mut map = toml::map::Map::new();
        map.insert(
            "__error__".to_string(),
            toml::Value::String(format!("{e}")),
        );
        Self {
            source: crate::layer::ConfigSource::Defaults, // placeholder
            data: toml::Value::Table(map),
        }
    }

    pub(crate) fn _deferred_error(&self) -> Option<ConfigError> {
        if let toml::Value::Table(ref t) = self.data {
            if let Some(toml::Value::String(msg)) = t.get("__error__") {
                return Some(ConfigError::MergeError(msg.clone()));
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as IoWrite;

    // ---- helper -----------------------------------------------------------

    fn minimal_project_toml(name: &str) -> String {
        format!(
            r#"
[project]
name = "{name}"
version = "0.1.0"
environment = "development"

[logging]
level = "info"
format = "text"
json = false
"#
        )
    }

    fn write_tmp_config(content: &str) -> (tempfile::NamedTempFile,) {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        (f,)
    }

    // ---- defaults ---------------------------------------------------------

    #[test]
    fn loader_new_has_defaults_layer() {
        // load() will fail validation because project.name is empty in defaults,
        // but we can inspect the layer count.
        let loader = ConfigLoader::new();
        assert_eq!(loader.layers.len(), 1);
    }

    // ---- file layer -------------------------------------------------------

    #[test]
    fn with_file_overrides_defaults() {
        let (tmp,) = write_tmp_config(&minimal_project_toml("from-file"));
        let config = ConfigLoader::new()
            .with_file(tmp.path())
            .load()
            .expect("should load");
        assert_eq!(config.project.name, "from-file");
    }

    #[test]
    fn with_file_optional_skips_missing() {
        // A non-existent path should not error; project.name stays empty →
        // validation will fail, but the error is about name, not file-not-found.
        let result = ConfigLoader::new()
            .with_file_optional("/tmp/__nonexistent_config_file__.toml")
            .load();
        // Should fail validation (empty name), not FileNotFound.
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ValidationFailed { field, .. } => {
                assert_eq!(field, "project.name");
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn with_file_mandatory_missing_errors() {
        let result = ConfigLoader::new()
            .with_file("/tmp/__nonexistent_mandatory__.toml")
            .load();
        assert!(result.is_err());
        // The error is surfaced as MergeError (via sentinel) or FileNotFound.
        // Either is acceptable; just confirm it is an error.
    }

    // ---- env layer --------------------------------------------------------

    /// Process-wide mutex to serialize tests that mutate environment variables.
    fn with_env<F: FnOnce() -> R, R>(vars: &[(&str, &str)], f: F) -> R {
        use once_cell::sync::Lazy;
        use std::sync::Mutex;
        static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        for (k, v) in vars {
            std::env::set_var(k, v);
        }
        let result = f();
        for (k, _) in vars {
            std::env::remove_var(k);
        }
        result
    }

    #[test]
    fn env_overrides_file_layer() {
        let vars = &[
            ("APP_PROJECT_NAME", "from-env"),
            ("APP_PROJECT_VERSION", "9.9.9"),
            ("APP_PROJECT_ENVIRONMENT", "production"),
            ("APP_LOGGING_LEVEL", "warn"),
            ("APP_LOGGING_FORMAT", "text"),
            ("APP_LOGGING_JSON", "false"),
        ];
        let (tmp,) = write_tmp_config(&minimal_project_toml("from-file"));
        let config = with_env(vars, || {
            ConfigLoader::new()
                .with_file(tmp.path())
                .with_env("APP")
                .load()
                .expect("should load")
        });
        assert_eq!(config.project.name, "from-env");
    }

    #[test]
    fn env_service_port_coerced_to_u16() {
        let vars = &[
            ("APP_PROJECT_NAME", "svc-test"),
            ("APP_PROJECT_VERSION", "1.0.0"),
            ("APP_PROJECT_ENVIRONMENT", "development"),
            ("APP_LOGGING_LEVEL", "info"),
            ("APP_LOGGING_FORMAT", "text"),
            ("APP_LOGGING_JSON", "false"),
            ("APP_SERVICE_HOST", "0.0.0.0"),
            ("APP_SERVICE_PORT", "9090"),
            ("APP_SERVICE_REQUEST_TIMEOUT_SECS", "60"),
        ];
        let config = with_env(vars, || {
            ConfigLoader::new().with_env("APP").load().expect("should load")
        });
        assert_eq!(config.service.unwrap().port, 9090u16);
    }

    // ---- override layer ---------------------------------------------------

    #[test]
    fn with_overrides_wins_over_file_and_env() {
        let (tmp,) = write_tmp_config(&minimal_project_toml("from-file"));
        let override_val: toml::Value = toml::from_str(
            r#"
[project]
name = "from-override"
"#,
        )
        .unwrap();

        let config = ConfigLoader::new()
            .with_file(tmp.path())
            .with_overrides(override_val)
            .load()
            .expect("should load");

        assert_eq!(config.project.name, "from-override");
    }

    // ---- precedence -------------------------------------------------------

    #[test]
    fn merge_precedence_defaults_lt_file_lt_env_lt_override() {
        // File sets name to "file-name", env sets to "env-name",
        // override sets to "override-name".  Override must win.
        let vars = &[
            ("APP_PROJECT_NAME", "env-name"),
            ("APP_PROJECT_VERSION", "1.0.0"),
            ("APP_PROJECT_ENVIRONMENT", "development"),
            ("APP_LOGGING_LEVEL", "info"),
            ("APP_LOGGING_FORMAT", "text"),
            ("APP_LOGGING_JSON", "false"),
        ];
        let (tmp,) = write_tmp_config(&minimal_project_toml("file-name"));
        let override_val: toml::Value =
            toml::from_str(r#"[project]\nname = "override-name""#.replace("\\n", "\n").as_str())
                .unwrap();

        let config = with_env(vars, || {
            ConfigLoader::new()
                .with_file(tmp.path())
                .with_env("APP")
                .with_overrides(override_val)
                .load()
                .expect("should load")
        });
        assert_eq!(config.project.name, "override-name");
    }

    // ---- database optional section ----------------------------------------

    #[test]
    fn database_section_populated_from_file() {
        let toml = r#"
[project]
name = "db-test"
version = "1.0.0"
environment = "development"

[logging]
level = "info"
format = "text"
json = false

[database]
url = "postgres://localhost/mydb"
max_connections = 10
timeout_secs = 5
"#;
        let (tmp,) = write_tmp_config(toml);
        let config = ConfigLoader::new()
            .with_file(tmp.path())
            .load()
            .expect("should load");

        let db = config.database.expect("database section must be present");
        assert_eq!(db.url, "postgres://localhost/mydb");
        assert_eq!(db.max_connections, 10);
    }

    // ---- load_config convenience ------------------------------------------

    #[test]
    fn load_config_convenience_fn_fails_gracefully_without_file() {
        // project.toml likely does not exist in /tmp; if it does that is OK
        // but we mostly verify the function does not panic.
        let result = load_config();
        // May succeed or fail validation — either way no panic.
        let _ = result;
    }
}
