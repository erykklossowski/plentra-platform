use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::Utc;
use serde_json::Value;

use crate::fetchers::stooq;
use crate::models::fuel::FuelData;
use crate::AppState;

const CACHE_KEY: &str = "fuels";

pub async fn handler(State(state): State<Arc<AppState>>) -> (HeaderMap, Json<Value>) {
    let mut headers = HeaderMap::new();
    headers.insert("cache-control", "max-age=900".parse().unwrap());

    // Check cache
    if let Some(cached) = state.cache.get(CACHE_KEY) {
        return (headers, Json(cached.data));
    }

    // Fetch all three in parallel
    let (ttf_res, ara_res, eua_res) = tokio::join!(
        stooq::fetch_ttf(&state.http_client),
        stooq::fetch_ara(&state.http_client),
        stooq::fetch_eua(&state.http_client),
    );

    match (ttf_res, ara_res, eua_res) {
        (Ok(ttf), Ok(ara), Ok(eua)) => {
            let fuel_data = FuelData {
                ttf_eur_mwh: ttf.current_price,
                ttf_change_pct: ttf.change_pct,
                ttf_history_30d: ttf.history_30d,
                ara_usd_tonne: ara.current_price,
                ara_change_pct: ara.change_pct,
                ara_history_30d: ara.history_30d,
                eua_eur_tonne: eua.current_price,
                eua_change_pct: eua.change_pct,
                eua_history_30d: eua.history_30d,
                fetched_at: Utc::now().to_rfc3339(),
                stale: None,
            };

            let value = serde_json::to_value(&fuel_data).unwrap();
            state
                .cache
                .set(CACHE_KEY.to_string(), value.clone(), state.config.cache_ttl_fuels);

            (headers, Json(value))
        }
        _ => {
            // At least one fetch failed — try stale cache
            if let Some(stale) = state.cache.get_stale(CACHE_KEY) {
                let mut data = stale.data;
                if let Some(obj) = data.as_object_mut() {
                    obj.insert("stale".to_string(), Value::Bool(true));
                }
                (headers, Json(data))
            } else {
                (
                    headers,
                    Json(serde_json::json!({
                        "error": "Failed to fetch fuel data and no cache available",
                        "timestamp": Utc::now().to_rfc3339()
                    })),
                )
            }
        }
    }
}
