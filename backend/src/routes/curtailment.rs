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

    // Extract curtailment data from cached residual response
    if let Some(cached) = state.cache.get("residual").or_else(|| state.cache.get_stale("residual"))
    {
        let data = &cached.data;
        let curtailment = json!({
            "ytd_curtailment_gwh": data.get("ytd_curtailment_gwh"),
            "forecast_curtailment_gwh": data.get("forecast_curtailment_gwh"),
            "wind_reduction_gwh": data.get("wind_reduction_gwh"),
            "solar_reduction_gwh": data.get("solar_reduction_gwh"),
            "fetched_at": data.get("fetched_at"),
        });
        return (headers, Json(curtailment));
    }

    (
        headers,
        Json(json!({
            "error": "No residual data available. Fetch /api/residual first.",
            "timestamp": Utc::now().to_rfc3339()
        })),
    )
}
