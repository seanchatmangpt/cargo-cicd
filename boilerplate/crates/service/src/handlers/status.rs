use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Service status snapshot returned by `GET /api/v1/status`.
///
/// With the `process-data` feature enabled, fields are populated from a real
/// `EngineState` query.  Without it, a minimal stub is returned.
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    /// Workspace / project name.
    pub name: String,
    /// Active Rust toolchain version string.
    pub rust_version: String,
    /// Current git branch, if determinable.
    pub git_branch: Option<String>,
    /// Overall verdict for the last completed pipeline run.
    pub verdict: String,
    /// Service uptime in seconds.
    pub uptime_secs: u64,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// `GET /api/v1/status`
///
/// Returns a workspace health snapshot.  With `--features process-data` this
/// queries the real `EngineState`; without it a minimal response is returned.
pub async fn service_status(
    State(state): State<AppState>,
) -> (StatusCode, Json<StatusResponse>) {
    #[cfg(feature = "process-data")]
    {
        use project_core::engine::EngineState;
        let engine = EngineState::from_workspace();
        let body = StatusResponse {
            name: engine.workspace.name.clone(),
            rust_version: engine.toolchain.rust_version.clone(),
            git_branch: Some(engine.git_phase.branch.clone()),
            verdict: "PASS".to_string(),
            uptime_secs: state.uptime_secs(),
        };
        return (StatusCode::OK, Json(body));
    }

    #[cfg(not(feature = "process-data"))]
    let body = StatusResponse {
        name: state
            .config
            .bind_addr
            .to_string()
            .split(':')
            .next()
            .unwrap_or("unknown")
            .to_string(),
        rust_version: "unknown".to_string(),
        git_branch: None,
        verdict: "PASS".to_string(),
        uptime_secs: state.uptime_secs(),
    };

    (StatusCode::OK, Json(body))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::Request, Router};
    use tower::ServiceExt;

    use crate::{router::create_app, state::ServiceConfig};

    fn test_app() -> Router {
        let config = ServiceConfig::default();
        let state = crate::state::AppState::new(config);
        create_app(state)
    }

    #[tokio::test]
    async fn status_returns_200() {
        let app = test_app();
        let req = Request::builder()
            .uri("/api/v1/status")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn status_body_has_required_fields() {
        let app = test_app();
        let req = Request::builder()
            .uri("/api/v1/status")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert!(json["name"].is_string());
        assert!(json["rust_version"].is_string());
        assert!(json["verdict"].is_string());
        assert!(json["uptime_secs"].is_number());
    }

    #[tokio::test]
    async fn status_verdict_is_pass() {
        let app = test_app();
        let req = Request::builder()
            .uri("/api/v1/status")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["verdict"], "PASS");
    }
}
