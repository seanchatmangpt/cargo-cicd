use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;
use uuid::Uuid;

/// Runtime item stored in the in-memory store (used by handlers/api.rs).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub created_at: String,
}

/// Configuration for the HTTP service, sourced from environment variables.
#[derive(Clone, Debug)]
pub struct ServiceConfig {
    /// Address to listen on, e.g. `0.0.0.0:8080`.
    pub bind_addr: SocketAddr,
    /// Per-request timeout in seconds.
    pub request_timeout_secs: u64,
    /// Allowed CORS origins.  An empty list disables the permissive policy.
    pub cors_origins: Vec<String>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:8080".parse().expect("default bind addr is valid"),
            request_timeout_secs: 30,
            cors_origins: vec![],
        }
    }
}

impl ServiceConfig {
    /// Construct a `ServiceConfig` from environment variables.
    ///
    /// | Variable                | Default          |
    /// |-------------------------|------------------|
    /// | `BIND_ADDR`             | `0.0.0.0:8080`   |
    /// | `REQUEST_TIMEOUT_SECS`  | `30`             |
    /// | `CORS_ORIGINS`          | `""`             |
    ///
    /// `CORS_ORIGINS` is a comma-separated list of origin strings.
    pub fn from_env() -> Self {
        let bind_addr = std::env::var("BIND_ADDR")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| "0.0.0.0:8080".parse().expect("fallback addr is valid"));

        let request_timeout_secs = std::env::var("REQUEST_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let cors_origins = std::env::var("CORS_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        Self {
            bind_addr,
            request_timeout_secs,
            cors_origins,
        }
    }
}

/// Shared application state injected into every handler via `axum::extract::State`.
///
/// All fields are cheap to clone (wrapped in `Arc`).
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ServiceConfig>,
    /// Monotonic instant at which the service started; used for uptime calculation.
    pub started_at: Instant,
    /// In-memory item store.  Keyed by item `Uuid`.
    pub items: Arc<RwLock<HashMap<Uuid, Item>>>,
}

impl AppState {
    /// Create a new `AppState` from the given `ServiceConfig`.
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            config: Arc::new(config),
            started_at: Instant::now(),
            items: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Elapsed seconds since service start.
    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn default_config_is_valid() {
        let cfg = ServiceConfig::default();
        assert_eq!(cfg.bind_addr.port(), 8080);
        assert_eq!(cfg.request_timeout_secs, 30);
        assert!(cfg.cors_origins.is_empty());
    }

    #[test]
    fn from_env_overrides_bind_addr() {
        env::set_var("BIND_ADDR", "127.0.0.1:9000");
        env::set_var("REQUEST_TIMEOUT_SECS", "60");
        env::set_var("CORS_ORIGINS", "https://example.com, https://other.com");

        let cfg = ServiceConfig::from_env();

        assert_eq!(cfg.bind_addr.to_string(), "127.0.0.1:9000");
        assert_eq!(cfg.request_timeout_secs, 60);
        assert_eq!(cfg.cors_origins, vec!["https://example.com", "https://other.com"]);

        env::remove_var("BIND_ADDR");
        env::remove_var("REQUEST_TIMEOUT_SECS");
        env::remove_var("CORS_ORIGINS");
    }

    #[test]
    fn from_env_falls_back_to_defaults_when_vars_absent() {
        env::remove_var("BIND_ADDR");
        env::remove_var("REQUEST_TIMEOUT_SECS");
        env::remove_var("CORS_ORIGINS");

        let cfg = ServiceConfig::from_env();
        assert_eq!(cfg.bind_addr.port(), 8080);
        assert_eq!(cfg.request_timeout_secs, 30);
        assert!(cfg.cors_origins.is_empty());
    }

    #[test]
    fn app_state_uptime_is_non_decreasing() {
        let state = AppState::new(ServiceConfig::default());
        let t0 = state.uptime_secs();
        let t1 = state.uptime_secs();
        assert!(t1 >= t0);
    }

    #[test]
    fn app_state_clone_shares_items() {
        let state = AppState::new(ServiceConfig::default());
        let clone = state.clone();
        // Both arcs point to the same allocation.
        assert!(Arc::ptr_eq(&state.items, &clone.items));
    }
}
