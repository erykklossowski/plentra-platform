use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

pub async fn handler() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "timestamp": Utc::now().to_rfc3339()
    }))
}
