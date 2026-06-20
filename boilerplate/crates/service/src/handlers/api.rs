use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ServiceError, state::AppState};

// Re-export Item so router can see it without going through state directly.
pub use crate::state::Item;

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

/// Request body for `POST /api/v1/items`.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateItemRequest {
    /// Item display name.  Must be 1–200 characters.
    pub name: String,
}

impl CreateItemRequest {
    /// Validate the request body.  Returns `Err(ServiceError::BadRequest)` on
    /// constraint violations.
    pub fn validate(&self) -> Result<(), ServiceError> {
        if self.name.trim().is_empty() {
            return Err(ServiceError::BadRequest {
                message: "`name` must not be empty".to_string(),
            });
        }
        if self.name.len() > 200 {
            return Err(ServiceError::BadRequest {
                message: format!(
                    "`name` must be at most 200 characters, got {}",
                    self.name.len()
                ),
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/items`
///
/// Returns all items in the in-memory store as a JSON array, ordered by
/// creation time (insertion order is preserved by iteration of sorted keys).
pub async fn list_items(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ServiceError> {
    let store = state.items.read().await;
    let mut items: Vec<Item> = store.values().cloned().collect();
    // Deterministic ordering by `created_at` string (ISO 8601 sorts lexicographically).
    items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok((StatusCode::OK, Json(items)))
}

/// `POST /api/v1/items`
///
/// Creates a new item and returns it with HTTP 201.
pub async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<CreateItemRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    payload.validate()?;

    let item = Item {
        id: Uuid::new_v4(),
        name: payload.name.trim().to_string(),
        created_at: chrono_or_fallback(),
    };

    let mut store = state.items.write().await;
    store.insert(item.id, item.clone());

    Ok((StatusCode::CREATED, Json(item)))
}

/// `GET /api/v1/items/:id`
///
/// Returns a single item or 404 if it does not exist.
pub async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let store = state.items.read().await;
    let item = store.get(&id).cloned().ok_or(ServiceError::NotFound)?;
    Ok((StatusCode::OK, Json(item)))
}

/// `DELETE /api/v1/items/:id`
///
/// Removes an item and returns 204.  Returns 404 if the item does not exist.
pub async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let mut store = state.items.write().await;
    if store.remove(&id).is_none() {
        return Err(ServiceError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Returns an ISO 8601 timestamp string.  Uses `std` only so the crate stays
/// dependency-light; replace with `jiff` or `chrono` if you need precision.
fn chrono_or_fallback() -> String {
    // std::time gives us seconds since UNIX epoch.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Format as a rough ISO 8601 approximation without an external crate.
    let (y, mo, d, h, mi, s) = epoch_to_ymdhms(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// Minimal epoch → (year, month, day, hour, min, sec) decomposition.
/// Handles dates from 1970 onwards.  Not leap-second-aware.
fn epoch_to_ymdhms(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let s = secs % 60;
    let total_min = secs / 60;
    let mi = total_min % 60;
    let total_hrs = total_min / 60;
    let h = total_hrs % 24;
    let total_days = total_hrs / 24;

    // Gregorian calendar approximation.
    let mut year = 1970u64;
    let mut days = total_days;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let month_days: [u64; 12] = [
        31,
        if is_leap(year) { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut month = 0usize;
    for mlen in &month_days {
        if days < *mlen {
            break;
        }
        days -= mlen;
        month += 1;
    }
    (year, month as u64 + 1, days + 1, h, mi, s)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::{Method, Request},
        Router,
    };
    use tower::ServiceExt;

    use crate::{router::create_app, state::ServiceConfig};

    fn test_app() -> Router {
        let config = ServiceConfig::default();
        let state = crate::state::AppState::new(config);
        create_app(state)
    }

    // -----------------------------------------------------------------------
    // Helper
    // -----------------------------------------------------------------------

    async fn body_json(resp: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).expect("valid json body")
    }

    // -----------------------------------------------------------------------
    // List
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn list_items_empty_returns_empty_array() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/items")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200);
        let json = body_json(resp).await;
        assert!(json.as_array().unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // Create
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn create_item_returns_201_with_id() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/items")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name":"hello"}"#))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 201);
        let json = body_json(resp).await;
        assert!(json["id"].is_string());
        assert_eq!(json["name"], "hello");
        assert!(json["created_at"].is_string());
    }

    #[tokio::test]
    async fn create_item_rejects_empty_name() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/items")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name":""}"#))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 400);
        let json = body_json(resp).await;
        assert_eq!(json["code"], "BAD_REQUEST");
    }

    #[tokio::test]
    async fn create_item_rejects_name_over_200_chars() {
        let long_name = "x".repeat(201);
        let payload = serde_json::json!({ "name": long_name }).to_string();
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/items")
            .header("content-type", "application/json")
            .body(Body::from(payload))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 400);
    }

    #[tokio::test]
    async fn create_item_trims_whitespace() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/items")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name":"  trimmed  "}"#))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 201);
        let json = body_json(resp).await;
        assert_eq!(json["name"], "trimmed");
    }

    // -----------------------------------------------------------------------
    // Get by id
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn get_item_returns_created_item() {
        let config = ServiceConfig::default();
        let state = crate::state::AppState::new(config);
        let app = create_app(state.clone());

        // Create via HTTP.
        let create_req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/items")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name":"fetch-me"}"#))
            .unwrap();
        let create_resp = app.clone().oneshot(create_req).await.unwrap();
        let create_json = body_json(create_resp).await;
        let id = create_json["id"].as_str().unwrap().to_string();

        // Fetch by id.
        let get_req = Request::builder()
            .method(Method::GET)
            .uri(format!("/api/v1/items/{id}"))
            .body(Body::empty())
            .unwrap();
        let get_resp = app.oneshot(get_req).await.unwrap();
        assert_eq!(get_resp.status(), 200);
        let get_json = body_json(get_resp).await;
        assert_eq!(get_json["id"], id);
        assert_eq!(get_json["name"], "fetch-me");
    }

    #[tokio::test]
    async fn get_item_returns_404_for_unknown_id() {
        let app = test_app();
        let unknown = uuid::Uuid::new_v4();
        let req = Request::builder()
            .method(Method::GET)
            .uri(format!("/api/v1/items/{unknown}"))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 404);
        let json = body_json(resp).await;
        assert_eq!(json["code"], "NOT_FOUND");
    }

    // -----------------------------------------------------------------------
    // Delete
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn delete_item_returns_204() {
        let config = ServiceConfig::default();
        let state = crate::state::AppState::new(config);
        let app = create_app(state.clone());

        // Create.
        let create_req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/items")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"name":"delete-me"}"#))
            .unwrap();
        let create_resp = app.clone().oneshot(create_req).await.unwrap();
        let create_json = body_json(create_resp).await;
        let id = create_json["id"].as_str().unwrap().to_string();

        // Delete.
        let del_req = Request::builder()
            .method(Method::DELETE)
            .uri(format!("/api/v1/items/{id}"))
            .body(Body::empty())
            .unwrap();
        let del_resp = app.oneshot(del_req).await.unwrap();
        assert_eq!(del_resp.status(), 204);
    }

    #[tokio::test]
    async fn delete_item_returns_404_for_unknown_id() {
        let app = test_app();
        let unknown = uuid::Uuid::new_v4();
        let req = Request::builder()
            .method(Method::DELETE)
            .uri(format!("/api/v1/items/{unknown}"))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 404);
    }

    // -----------------------------------------------------------------------
    // List after mutations
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn list_returns_items_after_creation() {
        let config = ServiceConfig::default();
        let state = crate::state::AppState::new(config);
        let app = create_app(state.clone());

        // Create two items.
        for name in ["alpha", "beta"] {
            let req = Request::builder()
                .method(Method::POST)
                .uri("/api/v1/items")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"name":"{name}"}}"#)))
                .unwrap();
            app.clone().oneshot(req).await.unwrap();
        }

        let list_req = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/items")
            .body(Body::empty())
            .unwrap();
        let list_resp = app.oneshot(list_req).await.unwrap();
        assert_eq!(list_resp.status(), 200);
        let json = body_json(list_resp).await;
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Fallback 404
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let app = test_app();
        let req = Request::builder()
            .uri("/does/not/exist")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 404);
    }

    // -----------------------------------------------------------------------
    // Validation unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn validate_rejects_whitespace_only_name() {
        let req = super::CreateItemRequest { name: "   ".to_string() };
        assert!(req.validate().is_err());
    }

    #[test]
    fn validate_accepts_max_length_name() {
        let req = super::CreateItemRequest { name: "x".repeat(200) };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn validate_rejects_name_over_200_chars() {
        let req = super::CreateItemRequest { name: "x".repeat(201) };
        assert!(req.validate().is_err());
    }
}
