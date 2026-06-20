use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Response body for `GET /health`.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
}

/// Response body for `GET /health/ready`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /health`
///
/// Returns service version and uptime.  Always 200 as long as the process is
/// running; use `/health/ready` for Kubernetes readiness probes.
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let body = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: state.uptime_secs(),
    };
    (StatusCode::OK, Json(body))
}

/// `GET /health/ready`
///
/// Kubernetes readiness probe.  Returns 200 when the service is ready to
/// accept traffic, 503 otherwise.  Extend this with actual readiness checks
/// (database ping, config validation, …) as the service matures.
pub async fn health_ready(State(_state): State<AppState>) -> impl IntoResponse {
    // Replace this stub with real readiness checks when you have dependencies.
    let ready = true;
    if ready {
        let body = ReadinessResponse { ready: true, reason: None };
        (StatusCode::OK, Json(body)).into_response()
    } else {
        let body = ReadinessResponse {
            ready: false,
            reason: Some("database unavailable".to_string()),
        };
        (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response()
    }
}

/// `GET /health/live`
///
/// Kubernetes liveness probe.  Always returns 200 — if the process is dead
/// the container runtime will never receive a response at all.
pub async fn health_live() -> impl IntoResponse {
    StatusCode::OK
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
    async fn health_returns_200() {
        let app = test_app();
        let req = Request::builder()
            .uri("/health")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn health_body_contains_ok_status() {
        let app = test_app();
        let req = Request::builder()
            .uri("/health")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["status"], "ok");
        assert!(json["uptime_secs"].is_number());
        assert!(json["version"].is_string());
    }

    #[tokio::test]
    async fn health_ready_returns_200() {
        let app = test_app();
        let req = Request::builder()
            .uri("/health/ready")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn health_live_returns_200() {
        let app = test_app();
        let req = Request::builder()
            .uri("/health/live")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200);
    }
}
