use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde_json::json;

use crate::{
    handlers::{
        api::{create_item, delete_item, get_item, list_items},
        health::{health_check, health_live, health_ready},
        status::service_status,
    },
    middleware::build_middleware_stack,
    state::AppState,
};

// ---------------------------------------------------------------------------
// Fallback
// ---------------------------------------------------------------------------

async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "route not found", "code": "NOT_FOUND" })),
    )
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build the complete `Router` with all routes and middleware applied.
///
/// Topology:
/// ```text
/// /health          — GET  → health_check
/// /health/ready    — GET  → health_ready
/// /health/live     — GET  → health_live
/// /api/v1/status   — GET  → service_status
/// /api/v1/items    — GET  → list_items
///                  — POST → create_item
/// /api/v1/items/:id— GET  → get_item
///                  — DELETE → delete_item
/// /* (fallback)    —      → 404 JSON
/// ```
pub fn create_router(state: AppState) -> Router {
    let health_routes = Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(health_ready))
        .route("/health/live", get(health_live));

    let api_v1_routes = Router::new()
        .route("/api/v1/status", get(service_status))
        .route("/api/v1/items", get(list_items).post(create_item))
        .route(
            "/api/v1/items/:id",
            get(get_item).delete(delete_item),
        );

    Router::new()
        .merge(health_routes)
        .merge(api_v1_routes)
        .fallback(not_found)
        .with_state(state.clone())
        .layer(build_middleware_stack(&state.config))
}

/// Alias used by `lib.rs` re-export and tests.
pub fn create_app(state: AppState) -> Router {
    create_router(state)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::{Method, Request},
    };
    use tower::ServiceExt;

    use super::*;
    use crate::state::ServiceConfig;

    fn test_router() -> Router {
        create_app(AppState::new(ServiceConfig::default()))
    }

    #[tokio::test]
    async fn get_health_200() {
        let router = test_router();
        let resp = router
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn get_health_ready_200() {
        let router = test_router();
        let resp = router
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn get_health_live_200() {
        let router = test_router();
        let resp = router
            .oneshot(
                Request::builder()
                    .uri("/health/live")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn get_api_v1_status_200() {
        let router = test_router();
        let resp = router
            .oneshot(
                Request::builder()
                    .uri("/api/v1/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn get_api_v1_items_200() {
        let router = test_router();
        let resp = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/items")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn unknown_route_returns_404_json() {
        let router = test_router();
        let resp = router
            .oneshot(
                Request::builder()
                    .uri("/does/not/exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 404);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["code"], "NOT_FOUND");
    }

    #[tokio::test]
    async fn post_items_and_get_back() {
        let state = AppState::new(ServiceConfig::default());
        let app = create_app(state);

        let create = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/items")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name":"router-test"}"#))
            .unwrap();
        let create_resp = app.clone().oneshot(create).await.unwrap();
        assert_eq!(create_resp.status(), 201);

        let bytes = to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
        let created: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let id = created["id"].as_str().unwrap();

        let get_req = Request::builder()
            .method(Method::GET)
            .uri(format!("/api/v1/items/{id}"))
            .body(Body::empty())
            .unwrap();
        let get_resp = app.oneshot(get_req).await.unwrap();
        assert_eq!(get_resp.status(), 200);
    }
}
