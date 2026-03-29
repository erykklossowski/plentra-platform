use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

pub async fn handler() -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=3600".parse().unwrap());

    (
        headers,
        Json(json!({
            "data_status": "unavailable",
            "message": "Price data not yet available. Historical prices endpoint coming soon.",
            "prices": [],
            "fetched_at": Utc::now().to_rfc3339(),
            "stale": true,
        })),
    )
}
