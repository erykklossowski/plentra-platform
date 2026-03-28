use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

use crate::AppState;

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=3600".parse().unwrap());

    // Extract reserve data from cached residual response
    if let Some(cached) = state.cache.get("residual").or_else(|| state.cache.get_stale("residual"))
    {
        let data = &cached.data;
        let reserves = json!({
            "current_residual_gw": data.get("current_residual_gw"),
            "must_run_floor_gw": data.get("must_run_floor_gw"),
            "stability_margin_gw": data.get("stability_margin_gw"),
            "congestion_probability_pct": data.get("congestion_probability_pct"),
            "fetched_at": data.get("fetched_at"),
        });
        return (headers, Json(reserves));
    }

    (
        headers,
        Json(json!({
            "error": "No residual data available. Fetch /api/residual first.",
            "timestamp": Utc::now().to_rfc3339()
        })),
    )
}
