use std::path::{Path, PathBuf};

use toml::Value;
use tracing::{debug, warn};

use crate::error::ConfigError;
use crate::schema::Config;

// ---------------------------------------------------------------------------
// ConfigSource — tracks where a layer came from (for diagnostics)
// ---------------------------------------------------------------------------

/// Records the origin of a [`ConfigLayer`] for logging and error reporting.
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// The compiled-in defaults (lowest priority).
    Defaults,
    /// A TOML file read from disk.
    File(PathBuf),
    /// Environment variables with a given prefix.
    Environment,
    /// A programmatic override value (highest priority when placed last).
    Override(Value),
}

// ---------------------------------------------------------------------------
// ConfigLayer
// ---------------------------------------------------------------------------

/// One source of configuration values.  A layer may be partial — keys that
/// are absent here simply inherit from lower-priority layers.
#[derive(Debug, Clone)]
pub struct ConfigLayer {
    /// Where these values came from.
    pub source: ConfigSource,
    /// The raw TOML value tree representing this layer's data.
    pub data: Value,
}

impl ConfigLayer {
    // ------------------------------------------------------------------
    // Construction
    // ------------------------------------------------------------------

    /// Build a layer from the compiled-in [`Config::default()`] values.
    ///
    /// This layer always succeeds and gives every key a known-good starting
    /// value; it should be the first (lowest-priority) layer in every stack.
    pub fn from_defaults() -> Self {
        let defaults = Config::default();
        // Serialize via TOML round-trip.  This cannot fail in practice because
        // the Default impl only produces TOML-serializable values.
        let toml_string =
            toml::to_string(&defaults).expect("Config::default() must be TOML-serializable");
        let data: Value =
            toml::from_str(&toml_string).expect("serialized defaults must be valid TOML");
        debug!("loaded defaults layer");
        Self {
            source: ConfigSource::Defaults,
            data,
        }
    }

    /// Read and parse a TOML config file.
    ///
    /// Returns [`ConfigError::FileNotFound`] when the path does not exist, or
    /// [`ConfigError::ParseError`] when the file is not valid TOML.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.to_path_buf()));
        }
        let raw = std::fs::read_to_string(path)?;
        let data: Value = toml::from_str(&raw).map_err(|e| ConfigError::ParseError {
            path: path.to_path_buf(),
            source: e,
        })?;
        debug!(path = %path.display(), "loaded file layer");
        Ok(Self {
            source: ConfigSource::File(path.to_path_buf()),
            data,
        })
    }

    /// Scan process environment variables whose names begin with `prefix`
    /// (followed by `_`) and build a partial TOML tree from them.
    ///
    /// ## Naming convention
    ///
    /// Variables are mapped to dotted TOML paths using the following rules:
    ///
    /// 1. Strip the `PREFIX_` prefix (case-insensitive match).
    /// 2. Split the remainder on `_` to produce path segments in lowercase.
    ///    Double-underscore (`__`) is treated as a segment separator to allow
    ///    values whose names contain a single underscore.
    /// 3. Nested segments produce nested TOML tables.
    ///
    /// Examples (prefix `"APP"`):
    /// ```text
    /// APP_LOGGING_LEVEL=debug   →  logging.level = "debug"
    /// APP_DATABASE_URL=...      →  database.url = "..."
    /// APP_SERVICE__PORT=9090    →  service.port = 9090  (integer)
    /// ```
    ///
    /// Env-var string values are automatically coerced to the most specific
    /// TOML scalar type: integer, float, boolean, or string (in that order).
    /// This means `"true"` → `bool`, `"9090"` → `i64`, `"3.14"` → `f64`.
    /// The serde deserialization step then maps TOML scalars to Rust types.
    pub fn from_env(prefix: &str) -> Self {
        let prefix_upper = format!("{}_", prefix.to_uppercase());
        let mut root: toml::map::Map<String, Value> = toml::map::Map::new();

        for (key, val) in std::env::vars() {
            if !key.to_uppercase().starts_with(&prefix_upper) {
                continue;
            }
            // Strip prefix (preserve original case in suffix for logging, but
            // work in lowercase for path construction).
            let suffix = key[prefix_upper.len()..].to_lowercase();

            // Split on __ first (explicit segment separator), then on _.
            let segments: Vec<&str> = suffix.split("__").collect();
            let path: Vec<String> = segments
                .iter()
                .flat_map(|s| s.split('_'))
                .map(|s| s.to_string())
                .collect();

            if path.is_empty() || path.iter().any(|s| s.is_empty()) {
                warn!(var = %key, "skipping env var: empty path segment");
                continue;
            }

            let typed_val = coerce_env_value(&val);
            debug!(var = %key, path = %path.join("."), value = %val, "env layer key");
            insert_nested(&mut root, &path, typed_val);
        }

        let data = Value::Table(root);
        Self {
            source: ConfigSource::Environment,
            data,
        }
    }

    /// Create a layer from an arbitrary [`toml::Value`].  Use this for
    /// programmatic overrides (e.g. CLI args after parsing).
    pub fn from_value(v: Value) -> Self {
        Self {
            source: ConfigSource::Override(v.clone()),
            data: v,
        }
    }
}

// ---------------------------------------------------------------------------
// Type coercion for env-var string values
// ---------------------------------------------------------------------------

/// Try to parse a raw env-var string into the most specific TOML scalar type.
///
/// Order of precedence:
/// 1. `"true"` / `"false"` (case-insensitive) → `Value::Boolean`
/// 2. Integer (i64) → `Value::Integer`
/// 3. Float (f64) → `Value::Float`
/// 4. Everything else → `Value::String`
///
/// This ensures that `APP_LOGGING_JSON=true` deserialises to `bool` rather
/// than failing with "expected bool, got string".
fn coerce_env_value(s: &str) -> Value {
    match s.to_lowercase().as_str() {
        "true" => return Value::Boolean(true),
        "false" => return Value::Boolean(false),
        _ => {}
    }
    if let Ok(i) = s.parse::<i64>() {
        return Value::Integer(i);
    }
    if let Ok(f) = s.parse::<f64>() {
        return Value::Float(f);
    }
    Value::String(s.to_string())
}

// ---------------------------------------------------------------------------
// Recursive insertion helper
// ---------------------------------------------------------------------------

/// Insert `value` at the dotted path `segments` inside `map`, creating
/// intermediate tables as needed.  Later calls overwrite earlier ones.
///
/// Works entirely with `toml::map::Map` to stay safe and avoid transmutes.
fn insert_nested(map: &mut toml::map::Map<String, Value>, segments: &[String], value: Value) {
    if segments.is_empty() {
        return;
    }
    if segments.len() == 1 {
        map.insert(segments[0].clone(), value);
        return;
    }
    // Descend (or create) a nested table.
    let head = segments[0].clone();
    let entry = map
        .entry(head)
        .or_insert_with(|| Value::Table(toml::map::Map::new()));
    if let Value::Table(ref mut child) = entry {
        insert_nested(child, &segments[1..], value);
    }
}

// ---------------------------------------------------------------------------
// Merge
// ---------------------------------------------------------------------------

/// Merge a slice of layers left-to-right (later layers win) and deserialise
/// the result into a [`Config`].
///
/// Only keys that are present in a later layer overwrite earlier values;
/// absent keys fall through unchanged.  Entire TOML tables are merged
/// recursively, not replaced wholesale.
pub fn merge_layers(layers: &[ConfigLayer]) -> Result<Config, ConfigError> {
    if layers.is_empty() {
        // No layers at all — return defaults without validation.
        return Ok(Config::default());
    }

    let mut base = layers[0].data.clone();
    for layer in &layers[1..] {
        merge_toml_values(&mut base, layer.data.clone());
    }

    let merged_str =
        toml::to_string(&base).map_err(|e| ConfigError::MergeError(e.to_string()))?;
    let config: Config =
        toml::from_str(&merged_str).map_err(|e| ConfigError::MergeError(e.to_string()))?;

    Ok(config)
}

/// Recursively merge `incoming` into `base`.  For tables, keys are merged
/// recursively.  For all other value types, `incoming` wins (overwrites).
fn merge_toml_values(base: &mut Value, incoming: Value) {
    match (base, incoming) {
        (Value::Table(ref mut b), Value::Table(i)) => {
            for (k, v) in i {
                match b.get_mut(&k) {
                    Some(existing) => merge_toml_values(existing, v),
                    None => {
                        b.insert(k, v);
                    }
                }
            }
        }
        (base, incoming) => *base = incoming,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // ---- from_defaults ----------------------------------------------------

    #[test]
    fn defaults_layer_produces_valid_toml() {
        let layer = ConfigLayer::from_defaults();
        // data must be a TOML table (not an array or scalar)
        assert!(matches!(layer.data, Value::Table(_)));
    }

    #[test]
    fn defaults_layer_has_logging_section() {
        let layer = ConfigLayer::from_defaults();
        let table = layer.data.as_table().unwrap();
        assert!(
            table.contains_key("logging"),
            "defaults layer must contain [logging]"
        );
    }

    // ---- from_env ---------------------------------------------------------

    /// Serialize all env-mutating tests through a process-wide mutex so they
    /// cannot interfere with each other when run in parallel.
    fn with_env_vars<F: FnOnce()>(vars: &[(&str, &str)], f: F) {
        use once_cell::sync::Lazy;
        use std::sync::Mutex;
        static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // Set
        for (k, v) in vars {
            env::set_var(k, v);
        }
        f();
        // Unset
        for (k, _) in vars {
            env::remove_var(k);
        }
    }

    #[test]
    fn env_layer_parses_logging_level() {
        with_env_vars(&[("APP_LOGGING_LEVEL", "debug")], || {
            let layer = ConfigLayer::from_env("APP");
            let table = layer.data.as_table().unwrap();
            let logging = table["logging"].as_table().unwrap();
            assert_eq!(logging["level"].as_str().unwrap(), "debug");
        });
    }

    #[test]
    fn env_layer_parses_service_port() {
        with_env_vars(&[("APP_SERVICE_PORT", "9090")], || {
            let layer = ConfigLayer::from_env("APP");
            let table = layer.data.as_table().unwrap();
            let service = table["service"].as_table().unwrap();
            // After coercion "9090" → Integer(9090).
            assert_eq!(service["port"].as_integer().unwrap(), 9090i64);
        });
    }

    #[test]
    fn env_layer_double_underscore_separator() {
        // APP_DATABASE__URL — two segments: database + url
        with_env_vars(&[("APP_DATABASE__URL", "postgres://localhost/test")], || {
            let layer = ConfigLayer::from_env("APP");
            let table = layer.data.as_table().unwrap();
            let db = table["database"].as_table().unwrap();
            assert_eq!(
                db["url"].as_str().unwrap(),
                "postgres://localhost/test"
            );
        });
    }

    #[test]
    fn env_layer_ignores_unrelated_vars() {
        with_env_vars(&[("UNRELATED_VAR", "irrelevant")], || {
            let layer = ConfigLayer::from_env("APP");
            let table = layer.data.as_table().unwrap();
            assert!(
                !table.contains_key("unrelated"),
                "unrelated env vars must not appear in layer"
            );
        });
    }

    #[test]
    fn env_layer_case_insensitive_prefix() {
        // Prefix comparison is done in uppercase, so "app" should work too.
        with_env_vars(&[("APP_LOGGING_JSON", "true")], || {
            let layer = ConfigLayer::from_env("app"); // lowercase prefix
            let table = layer.data.as_table().unwrap();
            let logging = table["logging"].as_table().unwrap();
            // After coercion "true" → Boolean(true).
            assert_eq!(logging["json"].as_bool().unwrap(), true);
        });
    }

    // ---- merge_layers -----------------------------------------------------

    #[test]
    fn merge_later_layer_wins() {
        let mut base = ConfigLayer::from_defaults();
        // Patch the logging level in the base
        if let Value::Table(ref mut t) = base.data {
            if let Some(Value::Table(ref mut l)) = t.get_mut("logging") {
                l.insert("level".to_string(), Value::String("error".to_string()));
            }
        }

        let override_toml: Value =
            toml::from_str(r#"[logging]\nlevel = "trace""#.replace("\\n", "\n").as_str())
                .unwrap();
        let override_layer = ConfigLayer::from_value(override_toml);

        let config = merge_layers(&[base, override_layer]).unwrap();
        assert_eq!(config.logging.level, "trace");
    }

    #[test]
    fn merge_absent_key_falls_through() {
        let defaults = ConfigLayer::from_defaults();
        // Override only the project name; logging should still come from defaults.
        let partial: Value =
            toml::from_str(r#"[project]\nname = "patched""#.replace("\\n", "\n").as_str())
                .unwrap();
        let override_layer = ConfigLayer::from_value(partial);

        let config = merge_layers(&[defaults, override_layer]).unwrap();
        assert_eq!(config.project.name, "patched");
        // Logging level should still be the default "info".
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn merge_empty_layers_returns_defaults() {
        let config = merge_layers(&[]).unwrap();
        assert_eq!(config.logging.level, "info");
    }
}
