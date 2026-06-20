use std::time::Duration;

use axum::http::{HeaderName, Request};
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

/// The canonical header name used to propagate a per-request identifier.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Returns a `SetRequestIdLayer` that generates a new UUID for every request
/// and stores it in the `x-request-id` header.
pub fn request_id_layer() -> SetRequestIdLayer<MakeRequestUuid> {
    let header = HeaderName::from_static(REQUEST_ID_HEADER);
    SetRequestIdLayer::new(header, MakeRequestUuid)
}

/// Returns a `PropagateRequestIdLayer` that copies the `x-request-id` from the
/// request into the response headers so clients can correlate logs.
pub fn propagate_request_id_layer() -> PropagateRequestIdLayer {
    let header = HeaderName::from_static(REQUEST_ID_HEADER);
    PropagateRequestIdLayer::new(header)
}

/// Returns a `TraceLayer` that logs every HTTP request/response at DEBUG level.
///
/// Spans include the HTTP method, path, status code, and latency.
pub fn trace_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    DefaultOnRequest,
    DefaultOnResponse,
> {
    TraceLayer::new_for_http()
        .on_request(DefaultOnRequest::new().level(Level::DEBUG))
        .on_response(DefaultOnResponse::new().level(Level::DEBUG))
}

/// Returns a `CorsLayer` configured from the supplied list of origins.
///
/// - If `origins` is empty, the permissive `Any` policy is applied (suitable
///   for local development; harden for production).
/// - Otherwise each entry is treated as an exact origin string.
pub fn cors_layer(origins: &[String]) -> CorsLayer {
    let layer = CorsLayer::new()
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any());

    if origins.is_empty() {
        layer.allow_origin(AllowOrigin::any())
    } else {
        let parsed: Vec<axum::http::HeaderValue> = origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        layer.allow_origin(parsed)
    }
}

/// Returns a `TimeoutLayer` that aborts requests that exceed `secs` seconds.
pub fn timeout_layer(secs: u64) -> TimeoutLayer {
    TimeoutLayer::new(Duration::from_secs(secs))
}

/// Composes the full middleware stack for a production-quality service.
///
/// Stack (outermost → innermost):
/// 1. `SetRequestId`        — generate `x-request-id` for every request
/// 2. `Trace`               — structured log span per request
/// 3. `Timeout`             — abort slow requests
/// 4. `Cors`                — CORS headers
/// 5. `PropagateRequestId`  — copy request id into response
///
/// Use the returned `ServiceBuilder` with `Router::layer(stack.into_inner())`.
pub fn build_middleware_stack(
    config: &crate::state::ServiceConfig,
) -> ServiceBuilder<
    tower::layer::util::Stack<
        PropagateRequestIdLayer,
        tower::layer::util::Stack<
            CorsLayer,
            tower::layer::util::Stack<
                TimeoutLayer,
                tower::layer::util::Stack<
                    TraceLayer<
                        tower_http::classify::SharedClassifier<
                            tower_http::classify::ServerErrorsAsFailures,
                        >,
                        DefaultOnRequest,
                        DefaultOnResponse,
                    >,
                    tower::layer::util::Stack<
                        SetRequestIdLayer<MakeRequestUuid>,
                        tower::layer::util::Identity,
                    >,
                >,
            >,
        >,
    >,
> {
    ServiceBuilder::new()
        .layer(request_id_layer())
        .layer(trace_layer())
        .layer(timeout_layer(config.request_timeout_secs))
        .layer(cors_layer(&config.cors_origins))
        .layer(propagate_request_id_layer())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Method, Request},
        response::IntoResponse,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    use super::*;
    use crate::state::ServiceConfig;

    fn make_app(config: &ServiceConfig) -> Router {
        let stack = build_middleware_stack(config);
        Router::new()
            .route("/ping", get(|| async { "pong".into_response() }))
            .layer(stack)
    }

    #[tokio::test]
    async fn middleware_stack_applies_without_panic() {
        let config = ServiceConfig::default();
        let app = make_app(&config);
        let req = Request::builder()
            .method(Method::GET)
            .uri("/ping")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn request_id_header_present_in_response() {
        let config = ServiceConfig::default();
        let app = make_app(&config);
        let req = Request::builder()
            .method(Method::GET)
            .uri("/ping")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        // The propagate layer copies x-request-id from request to response.
        // When no id was in the request the set-id layer generates one.
        assert!(resp.headers().contains_key(REQUEST_ID_HEADER));
    }

    #[tokio::test]
    async fn cors_any_origin_when_no_origins_configured() {
        let config = ServiceConfig {
            cors_origins: vec![],
            ..Default::default()
        };
        let layer = cors_layer(&config.cors_origins);
        // If construction didn't panic we're good; cors_layer returns a valid layer.
        let _ = layer;
    }

    #[tokio::test]
    async fn cors_specific_origin_does_not_panic() {
        let origins = vec!["https://example.com".to_string()];
        let layer = cors_layer(&origins);
        let _ = layer;
    }

    #[test]
    fn timeout_layer_constructed_from_config() {
        let layer = timeout_layer(10);
        let _ = layer; // Construction is the test.
    }
}
