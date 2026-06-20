use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// Body returned for every error response.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    /// Human-readable description.
    pub error: String,
    /// Machine-readable error code (upper-snake: `NOT_FOUND`, `BAD_REQUEST`, …).
    pub code: String,
    /// Forwarded request-id when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// All errors that the service can surface to HTTP clients.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("resource not found")]
    NotFound,

    #[error("bad request: {message}")]
    BadRequest { message: String },

    #[error("unauthorized")]
    Unauthorized,

    #[error("internal error: {source}")]
    Internal {
        #[from]
        source: anyhow::Error,
    },
}

impl ServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::NotFound => StatusCode::NOT_FOUND,
            ServiceError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ServiceError::Unauthorized => StatusCode::UNAUTHORIZED,
            ServiceError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            ServiceError::NotFound => "NOT_FOUND",
            ServiceError::BadRequest { .. } => "BAD_REQUEST",
            ServiceError::Unauthorized => "UNAUTHORIZED",
            ServiceError::Internal { .. } => "INTERNAL_SERVER_ERROR",
        }
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ErrorBody {
            error: self.to_string(),
            code: self.code().to_string(),
            request_id: None,
        };
        (status, Json(body)).into_response()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    async fn body_json(resp: Response) -> serde_json::Value {
        let bytes = to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("body bytes");
        serde_json::from_slice(&bytes).expect("json body")
    }

    #[tokio::test]
    async fn not_found_returns_404() {
        let resp = ServiceError::NotFound.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let json = body_json(resp).await;
        assert_eq!(json["code"], "NOT_FOUND");
    }

    #[tokio::test]
    async fn bad_request_returns_400_with_message() {
        let err = ServiceError::BadRequest {
            message: "field `name` is required".to_string(),
        };
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let json = body_json(resp).await;
        assert_eq!(json["code"], "BAD_REQUEST");
        assert!(json["error"].as_str().unwrap().contains("name"));
    }

    #[tokio::test]
    async fn unauthorized_returns_401() {
        let resp = ServiceError::Unauthorized.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let json = body_json(resp).await;
        assert_eq!(json["code"], "UNAUTHORIZED");
    }

    #[tokio::test]
    async fn internal_returns_500() {
        let err = ServiceError::Internal {
            source: anyhow::anyhow!("db exploded"),
        };
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let json = body_json(resp).await;
        assert_eq!(json["code"], "INTERNAL_SERVER_ERROR");
    }

    #[test]
    fn anyhow_converts_to_internal() {
        let err: ServiceError = anyhow::anyhow!("disk full").into();
        assert!(matches!(err, ServiceError::Internal { .. }));
    }
}
